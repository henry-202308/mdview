# 贡献指南

感谢你愿意改进 mdview。提交 Issue 或 Pull Request 前，请先阅读以下约定。

## 环境要求

- Windows 10 / 11 x64
- Rust 1.92+（`rustup` 安装）
- Visual Studio 2022 Build Tools，勾选「使用 C++ 的桌面开发」
- Microsoft Edge WebView2 Runtime

## 本地构建与测试

```powershell
# 自动定位 MSVC 与 cargo
.\build.bat build --release
.\build.bat test

# 或已在 x64 Native Tools Command Prompt 中
cargo build --release
cargo test
```

提交前请至少运行：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets
cargo test --all
```

## 代码风格

- 使用 `rustfmt` 默认配置，提交前跑 `cargo fmt --all`。
- 前端代码在 `assets/`，没有构建步骤，修改后重新 `cargo build` 即可嵌入。
- `assets/` 下的 `.min.js` 是上游压缩产物，除非升级版本不要手改。
- 新增或升级前端库时，同步更新 `THIRD_PARTY_NOTICES.md` 与 `licenses/`。

## 提交信息

用一句话说明改动，必要时补一行原因：

```text
fix: 修复 GB18030 文档另存后编码被改写
```

## Pull Request

- 一个 PR 只做一件事，便于 review。
- 新功能请附带测试，或写清手动验证步骤。
- 用户可见的变化请更新 `CHANGELOG.md`。
- 不要提交 `dist/`、`target/`、`*.WebView2`、`.mdview-data/` 等本地产物。

## 报告问题

请使用 Issue 模板，附上 mdview 版本、Windows 版本、复现步骤和截图。安全问题不要开公开 Issue，见 [SECURITY.md](SECURITY.md)。
