# MyKey

一个面向 `Windows / Android / iOS` 的 Flutter 新工程，密码算法使用纯 Dart 重写，目标行为对齐 `github.com/cloudflare/gokey` 的 `GetPass` 逻辑。

## 目录结构

```text
.
|-- lib/
|   |-- main.dart
|   `-- src/
|       `-- mykey_service.dart
|-- test/
|   |-- mykey_service_test.dart
|   `-- widget_test.dart
|-- windows/
|   |-- flutter/
|   |-- runner/
|   `-- CMakeLists.txt
|-- pubspec.yaml
`-- README.md
```

## 功能

- 主密码输入框
- 服务标识输入框
- 结果只读输入框
- 计算按钮
- 复制密码按钮

## 算法说明

Dart 实现位于 `lib/src/mykey_service.dart`，主要复刻了 gokey 的密码推导流程：

1. `PBKDF2-HMAC-SHA256(password, realm + "-pass", 4096, 32 bytes)`
2. 以结果作为 `AES-256-CTR` 的 key，使用全 0 的 16 字节计数器初值
3. 从生成的确定性字节流中按 gokey 的 `randRange` 规则取样字符
4. 持续尝试直到满足密码规则

默认密码规则：

- 长度 `16`
- 至少 `1` 个大写字母
- 至少 `1` 个小写字母
- 至少 `1` 个数字
- 至少 `1` 个特殊字符
- 允许的特殊字符：`!@#$%^&*()-_=+?`

## 本地开发

安装 Flutter 后可直接在仓库根目录执行：

```powershell
flutter pub get
flutter test
flutter run -d windows
```

如果已经安装 Android Studio 和 Android SDK，可先检查 Android 环境：

```powershell
flutter doctor
```

如需生成 Android APK，可在仓库根目录执行：

```powershell
flutter build apk --release
```

编译产物默认位于：

```text
build/app/outputs/flutter-apk/app-release.apk
```

如果需要调试版 APK，可执行：

```powershell
flutter build apk --debug
```

如果要运行 Android 或 iOS，再分别补齐对应平台工具链：

- Android Studio / Android SDK（Android APK 已可编译）
- Xcode（仅 macOS 可构建 iOS）

## 已知说明

- 现在主实现已经切换到 Flutter + 纯 Dart
- 当前实现不依赖 `gcc`、`cargo` 或 Go 动态库
- iOS 仍然只能在 macOS 上完成最终构建和签名
