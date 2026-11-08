# Changelog

本项目从 0.1.0 开始记录变更，格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [Unreleased]

## [0.1.0] - 2026-10-25

### Added

- 本地 Markdown 阅读，支持 `.md` `.markdown` `.mdown` `.mkd` `.mdx` `.txt`
- 多窗口：`Ctrl+N` 新建窗口，`Ctrl+Shift+O` 在新窗口打开
- F2 就地编辑（Turndown + GFM 往返）与 F3 源码分屏编辑，自动保存
- GitHub 风格 Markdown：表格、任务列表、删除线、脚注、提示块
- 文档目录、阅读进度、滚动高亮
- 离线 highlight.js 代码高亮与一键复制
- 离线 KaTeX 数学公式（MathML）与 Mermaid 图表
- 本地图片协议，不占用 HTTP 端口
- 深色 / 浅色主题与 8 套配色
- 字号、版心宽度、字体、目录开关等阅读设置
- 单文件 HTML 导出（图片 / 公式 / Mermaid 内嵌）
- 无对话框 PDF 导出（`Ctrl+P`）与系统打印
- UTF-8 / UTF-16 / GB18030 编码兼容
- 当前用户级 `.md` 文件关联（`--register` / `--unregister`）
- 最近文件列表

[Unreleased]: https://github.com/henry-202308/mdview/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/henry-202308/mdview/releases/tag/v0.1.0
