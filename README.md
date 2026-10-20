# MyKey

MyKey 根据主密码和服务标识，在本地生成确定性密码。Windows 桌面主入口已迁移为
**Rust + GPUI 0.2.2**，界面直接由 GPU 绘制，不依赖 Flutter 或 WebView。
当前仓库的旧实现实际是 Flutter，并非 Tauri。

## Windows 开发

需要完整的 Rust **MSVC** 工具链、Visual Studio 2022 的「使用 C++ 的桌面开发」
工作负载与 Windows SDK，以及支持 DirectX 11 的图形环境。

```powershell
rustup toolchain install stable-x86_64-pc-windows-msvc --profile minimal
cargo +stable-x86_64-pc-windows-msvc run --locked
```

构建发布版：

```powershell
cargo +stable-x86_64-pc-windows-msvc build --release --locked
```

程序位于 `target/release/mykey.exe`。打包时附带 `LICENSE`、
`THIRD_PARTY_NOTICES.md` 和 `licenses/`。

本机如果沿用已有的 GNU Rust 工具链与 MSYS2 MinGW，也可以在当前 PowerShell 会话中指定
完整的 GCC（仅有 Rust 自带的链接器而缺少 MinGW 库时不能完成链接）：

```powershell
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = 'C:/msys64/mingw64/bin/gcc.exe'
$env:PATH = 'C:/msys64/mingw64/bin;' + $env:PATH
cargo run --locked
```

## 功能

- 中文深色界面，主密码默认隐藏，可切换显隐。
- 服务标识输入、只读结果、后台计算、一键复制与清空。
- 修改任意输入会清除旧结果，避免复制与当前输入不对应的密码。
- Enter / Ctrl+Enter 生成；Tab / Shift+Tab 切换输入框和按钮。
- 支持 Unicode 输入、选择、粘贴和输入法组合文本。
- 不向磁盘保存主密码和生成结果；复制后的密码保留在系统剪贴板中。

## 算法兼容性

Rust 实现位于 `src/lib.rs`，保留旧 Dart 版本行为：

1. UTF-8 输入，不裁剪空格、不改变大小写、不做 Unicode 归一化。
2. `PBKDF2-HMAC-SHA256(master, service + "-pass", 4096, 32 bytes)`。
3. AES-256-CTR，全零的 16 字节大端初始计数器，从字节流开头取样。
4. 保留 gokey 的拒绝取样和字符顺序，输出 16 位密码，至少含一个大写字母、
   小写字母、数字和特殊字符；允许的特殊字符为 `!@#$%^&*()-_=+?`。

密码兼容性测试可独立运行，不需要 GPUI 图形环境：

```powershell
cargo +stable-x86_64-pc-windows-msvc test --no-default-features --locked
cargo +stable-x86_64-pc-windows-msvc check --locked
cargo fmt --check
```

测试包含旧版三个 gokey 固定向量、空值校验、Unicode 与空格输入和密码规则。

## 目录

| 路径 | 用途 |
| --- | --- |
| `src/main.rs` | GPUI 桌面界面、后台任务、系统剪贴板 |
| `src/input.rs` | 原生文本输入、密码遮罩、选择与输入法 |
| `src/lib.rs` | 密码算法与兼容性测试 |
| `src/text_offsets.rs` | Unicode、Windows 输入法与密码遮罩位置转换 |
| `Cargo.toml` / `Cargo.lock` | Rust 入口与依赖锁定 |
| `lib/`、`test/`、`android/`、`windows/` | 保留的旧 Flutter 工程 |

## 移动端与旧 Flutter 工程

本次 GPUI 迁移针对桌面端。Android 继续使用保留的 Flutter 实现，发布工作流仍会构建
Android APK；Windows 发布改为构建 GPUI 程序。仓库目前没有 iOS 平台工程。

如需运行原 Flutter 工程：

```powershell
flutter pub get
flutter test
flutter run -d android
flutter build apk --release
```

旧 `windows/` 目录仅供 Flutter 回退使用，GPUI 构建不读取该目录。
