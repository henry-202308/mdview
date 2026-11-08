# 安全政策

## 支持范围

当前维护 `0.1.x` 系列，更早版本不再接收安全修复。

## 报告漏洞

请通过 GitHub 的私密漏洞报告提交：

1. 打开 <https://github.com/henry-202308/mdview/security/advisories/new>
2. 描述问题、复现步骤和影响范围

请勿在公开 Issue 中披露未修复的漏洞。我会在 7 天内确认收到，并在修复后于 Release Notes / CHANGELOG 中致谢（如你愿意）。

## 设计说明

- mdview 是本地桌面程序，不启动 HTTP 服务，不上传文档，不包含遥测。
- Markdown 中的 `<script>`、事件属性和 iframe 在插入 DOM 前会被清理，CSP 禁止 `connect-src`。
- 文档里显式引用的远程图片仍会按 URL 加载，这属于文档内容而非程序行为。
- 构建产物未签名，Windows SmartScreen 可能提示；请只从本仓库 Releases 下载。
