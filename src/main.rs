#![cfg_attr(windows, windows_subsystem = "windows")]

mod input;

use gpui::{prelude::*, *};
use input::{Changed, TextInput};
use zeroize::Zeroizing;

actions!(
    mykey,
    [Generate, CopyPassword, Clear, NextField, PreviousField]
);

struct MyKey {
    master: Entity<TextInput>,
    service: Entity<TextInput>,
    result: Entity<TextInput>,
    generate_focus: FocusHandle,
    copy_focus: FocusHandle,
    clear_focus: FocusHandle,
    status: SharedString,
    error: bool,
    busy: bool,
    _subscriptions: Vec<Subscription>,
}

impl MyKey {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let master = cx.new(|cx| TextInput::new("请输入主密码", true, cx));
        let service = cx.new(|cx| TextInput::new("例如 github.com", false, cx));
        let result = cx.new(|cx| {
            let mut input = TextInput::new("生成的密码将在这里显示", false, cx);
            input.readonly = true;
            input
        });
        let subscriptions = [&master, &service]
            .into_iter()
            .map(|field| {
                cx.subscribe(field, |this, _, _: &Changed, cx| {
                    this.result.update(cx, |input, cx| input.set_value("", cx));
                    this.status = "输入已更新，请生成密码".into();
                    this.error = false;
                    cx.notify();
                })
            })
            .collect();
        window.focus(&master.focus_handle(cx));
        Self {
            master,
            service,
            result,
            generate_focus: cx.focus_handle().tab_stop(true),
            copy_focus: cx.focus_handle().tab_stop(true),
            clear_focus: cx.focus_handle().tab_stop(true),
            status: "准备就绪".into(),
            error: false,
            busy: false,
            _subscriptions: subscriptions,
        }
    }

    fn generate(&mut self, _: &Generate, _: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.master.read(cx).is_composing() || self.service.read(cx).is_composing()
        {
            return;
        }
        let master = Zeroizing::new(self.master.read(cx).value().to_owned());
        let service = self.service.read(cx).value().to_owned();
        self.result.update(cx, |input, cx| input.set_value("", cx));
        if master.is_empty() || service.is_empty() {
            self.status = if master.is_empty() {
                "主密码不能为空"
            } else {
                "服务标识不能为空"
            }
            .into();
            self.error = true;
            cx.notify();
            return;
        }
        self.busy = true;
        self.error = false;
        self.status = "正在计算密码…".into();
        for field in [&self.master, &self.service] {
            field.update(cx, |input, cx| {
                input.disabled = true;
                cx.notify();
            });
        }
        let task = cx
            .background_executor()
            .spawn(async move { mykey::derive_password(&master, &service) });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |this, cx| {
                this.busy = false;
                for field in [&this.master, &this.service] {
                    field.update(cx, |input, cx| {
                        input.disabled = false;
                        cx.notify();
                    });
                }
                match result {
                    Ok(password) => {
                        let password = Zeroizing::new(password);
                        this.result
                            .update(cx, |input, cx| input.set_value(&password, cx));
                        this.status = "密码已生成，可以复制使用".into();
                        this.error = false;
                    }
                    Err(message) => {
                        this.status = message.into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn copy(&mut self, _: &CopyPassword, _: &mut Window, cx: &mut Context<Self>) {
        let value = self.result.read(cx).value();
        if value.is_empty() || self.busy {
            self.status = "请先生成密码".into();
            self.error = true;
        } else {
            cx.write_to_clipboard(ClipboardItem::new_string(value.to_owned()));
            self.status = "密码已复制到剪贴板".into();
            self.error = false;
        }
        cx.notify();
    }

    fn clear(&mut self, _: &Clear, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        for field in [&self.master, &self.service, &self.result] {
            field.update(cx, |input, cx| input.set_value("", cx));
        }
        self.master.update(cx, |input, cx| {
            input.masked = true;
            cx.notify();
        });
        self.status = "内容已清空".into();
        self.error = false;
        window.focus(&self.master.focus_handle(cx));
        cx.notify();
    }
}

fn label(text: &'static str) -> Div {
    div().text_sm().text_color(rgb(0xcbd5e1)).child(text)
}

fn button(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h_12()
        .px_4()
        .rounded_lg()
        .cursor_pointer()
        .border_1()
        .border_color(rgb(0x334155))
        .focus(|style| style.border_color(rgb(0x99f6e4)))
}

impl Render for MyKey {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let masked = self.master.read(cx).masked;
        let can_copy = !self.busy && !self.result.read(cx).value().is_empty();
        div()
            .id("mykey")
            .key_context("MyKey")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x07131f))
            .text_color(rgb(0xf1f5f9))
            .font_family("Microsoft YaHei UI")
            .text_base()
            .on_action(cx.listener(Self::generate))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::clear))
            .on_action(cx.listener(|_, _: &NextField, window, _| window.focus_next()))
            .on_action(cx.listener(|_, _: &PreviousField, window, _| window.focus_prev()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .p_8()
                    .gap_6()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(480.))
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x5eead4))
                                    .child("MYKEY / 密码生成器"),
                            )
                            .child(div().text_xs().text_color(rgb(0x94a3b8)).child("本地计算")),
                    )
                    .child(
                        div()
                            .w_full()
                            .max_w(px(480.))
                            .flex()
                            .flex_col()
                            .gap_5()
                            .p_6()
                            .rounded_2xl()
                            .bg(rgb(0x0d1b2c))
                            .border_1()
                            .border_color(rgb(0x203248))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_3xl()
                                            .font_weight(FontWeight::BOLD)
                                            .child("一个主密码。"),
                                    )
                                    .child(
                                        div()
                                            .text_3xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x5eead4))
                                            .child("每个服务，各不相同。"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x94a3b8))
                                            .child("使用主密码与服务标识，生成可重复的专属密码。"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .child(label("主密码"))
                                            .child(
                                                div()
                                                    .id("toggle-master")
                                                    .text_xs()
                                                    .text_color(rgb(0x5eead4))
                                                    .cursor_pointer()
                                                    .child(if masked {
                                                        "显示密码"
                                                    } else {
                                                        "隐藏密码"
                                                    })
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.master.update(cx, |input, cx| {
                                                            input.masked = !input.masked;
                                                            cx.notify();
                                                        });
                                                        cx.notify();
                                                    })),
                                            ),
                                    )
                                    .child(self.master.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(label("服务标识"))
                                    .child(self.service.clone())
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x94a3b8))
                                            .child("请保持拼写与大小写一致，空格也会影响结果。"),
                                    ),
                            )
                            .child(
                                button("generate")
                                    .key_context("GenerateButton")
                                    .track_focus(&self.generate_focus)
                                    .w_full()
                                    .bg(rgb(0x14b8a6))
                                    .text_color(rgb(0x042f2e))
                                    .font_weight(FontWeight::BOLD)
                                    .when(self.busy, |style| style.opacity(0.6))
                                    .hover(|style| style.bg(rgb(0x2dd4bf)))
                                    .child(if self.busy {
                                        "正在生成…"
                                    } else {
                                        "生成密码"
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.generate(&Generate, window, cx)
                                    })),
                            )
                            .child(
                                div()
                                    .border_t_1()
                                    .border_color(rgb(0x203248))
                                    .pt_5()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(label("生成结果"))
                                    .child(self.result.clone())
                                    .child(
                                        div()
                                            .flex()
                                            .gap_3()
                                            .child(
                                                button("copy")
                                                    .key_context("CopyButton")
                                                    .track_focus(&self.copy_focus)
                                                    .flex_1()
                                                    .bg(rgb(0x14283b))
                                                    .when(!can_copy, |style| style.opacity(0.4))
                                                    .child("复制密码")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.copy(&CopyPassword, window, cx)
                                                        },
                                                    )),
                                            )
                                            .child(
                                                button("clear")
                                                    .key_context("ClearButton")
                                                    .track_focus(&self.clear_focus)
                                                    .child("清空")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.clear(&Clear, window, cx)
                                                        },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x94a3b8))
                                            .child("16 位 · 大写字母 · 小写字母 · 数字 · 特殊字符"),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(if self.error { 0xfca5a5 } else { 0x99f6e4 }))
                                    .child(self.status.clone()),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .max_w(px(480.))
                            .flex()
                            .justify_between()
                            .text_xs()
                            .text_color(rgb(0x64748b))
                            .child("不保存主密码和生成结果")
                            .child("Ctrl + Enter 生成"),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        input::bind_keys(cx);
        cx.bind_keys([
            KeyBinding::new("ctrl-enter", Generate, Some("MyKey")),
            KeyBinding::new("enter", Generate, Some("TextInput")),
            KeyBinding::new("enter", Generate, Some("GenerateButton")),
            KeyBinding::new("enter", CopyPassword, Some("CopyButton")),
            KeyBinding::new("enter", Clear, Some("ClearButton")),
            KeyBinding::new("tab", NextField, Some("MyKey")),
            KeyBinding::new("shift-tab", PreviousField, Some("MyKey")),
        ]);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let bounds = Bounds::centered(None, size(px(600.), px(820.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(440.), px(560.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("MyKey".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| MyKey::new(window, cx)),
        )
        .expect("无法创建 MyKey 窗口");
        cx.activate(true);
    });
}
