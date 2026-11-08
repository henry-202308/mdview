use crate::settings::Settings;
use serde_json::json;

pub const APP_CSS: &str = include_str!("../assets/app.css");
pub const APP_JS: &str = include_str!("../assets/app.js");
pub const HIGHLIGHT_JS: &str = concat!(
    include_str!("../assets/highlight.min.js"),
    "\n",
    include_str!("../assets/highlight-powershell.min.js"),
    "\n",
    include_str!("../assets/highlight-dos.min.js")
);
pub const KATEX_JS: &str = include_str!("../assets/katex.min.js");
pub const AUTO_RENDER_JS: &str = include_str!("../assets/auto-render.min.js");
pub const MERMAID_JS: &str = include_str!("../assets/mermaid.min.js");
pub const TURNDOWN_JS: &str = include_str!("../assets/turndown.min.js");
pub const TURNDOWN_GFM_JS: &str = include_str!("../assets/turndown-plugin-gfm.min.js");

pub fn shell_html(
    settings: &Settings,
    recent: &[String],
    version: &str,
    webview_version: &str,
) -> String {
    let bootstrap = json!({
        "settings": settings,
        "recents": recent,
        "version": version,
        "webview": webview_version,
    });
    let bootstrap = serde_json::to_string(&bootstrap)
        .unwrap_or_else(|_| "{}".to_string())
        .replace("</", "<\\/");

    let mut html =
        String::with_capacity(APP_CSS.len() + APP_JS.len() + HIGHLIGHT_JS.len() + 90_000);
    html.push_str(
        r##"<!doctype html>
<html lang="zh-CN" data-theme="light" data-accent="github" data-font="sans">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self' mdview: data: http: https:; style-src 'unsafe-inline'; script-src 'unsafe-inline'; font-src data:; media-src 'self' mdview: data: https: http:; connect-src 'none';">
<meta name="theme-color" content="#f5f6f7">
<title>mdview — Markdown 阅读器</title>
<style>
"##,
    );
    html.push_str(APP_CSS);
    html.push_str(
        r##"
[hidden] { display: none !important; }
.text-btn { border: 0; background: none; color: var(--muted); font-size: 10.5px; cursor: pointer; padding: 2px 4px; }
.text-btn:hover { color: var(--accent); }
.empty-doc { color: var(--muted); }
.image-error { opacity: .55; }
</style>
</head>
<body class="empty">
<div id="progress"></div>

<header class="topbar">
  <div class="brand">
    <div class="brand-mark">M↓</div>
    <div class="brand-name">mdview <small>Rust</small></div>
  </div>
  <div class="doc-heading">
    <span id="doc-title">Markdown Reader</span>
    <span id="doc-path">拖入文件或按 Ctrl+O</span>
  </div>
  <div class="top-actions">
    <button id="btn-new" class="icon-btn" title="新建窗口 (Ctrl+N)" aria-label="新建窗口">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M12 5v14M5 12h14" stroke-linecap="round"/></svg>
    </button>
    <button id="btn-toc" class="icon-btn" title="显示/隐藏目录 (Ctrl+B)" aria-label="目录">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" stroke-linecap="round"/></svg>
    </button>
    <button id="btn-open" class="icon-btn" title="打开文件 (Ctrl+O)" aria-label="打开文件">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/></svg>
    </button>
    <button id="btn-theme" class="icon-btn" title="切换主题 (Ctrl+D)" aria-label="切换主题">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5 8.5 8.5 0 1 0 20.5 14.5Z" stroke-linejoin="round"/></svg>
    </button>
    <button id="btn-settings" class="icon-btn" title="阅读设置 (Ctrl+,)" aria-label="设置">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="3.2"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.83 2.83-.06-.06a1.7 1.7 0 0 0-1.88-.34 1.7 1.7 0 0 0-1.03 1.56V21h-4v-.08A1.7 1.7 0 0 0 8.96 19.4a1.7 1.7 0 0 0-1.88.34l-.06.06-2.83-2.83.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.56-1.03H3v-4h.08A1.7 1.7 0 0 0 4.6 8.94a1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.83-2.83.06.06A1.7 1.7 0 0 0 8.96 4.6 1.7 1.7 0 0 0 10 3.04V3h4v.08a1.7 1.7 0 0 0 1.03 1.56 1.7 1.7 0 0 0 1.88-.34l.06-.06 2.83 2.83-.06.06A1.7 1.7 0 0 0 19.4 9c.13.62.66 1.08 1.3 1.08H21v4h-.3c-.64 0-1.17.46-1.3 1.08Z" stroke-linejoin="round"/></svg>
    </button>
    <button id="btn-edit" class="icon-btn" title="就地编辑 (F2)" aria-label="就地编辑">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m4 20 4.5-1 10-10a2.1 2.1 0 0 0-3-3l-10 10L4 20Z" stroke-linejoin="round"/><path d="m13.5 7.5 3 3" stroke-linecap="round"/></svg>
    </button>
  </div>
</header>

<section id="welcome">
  <div class="welcome-card">
    <div class="hero">
      <div class="hero-logo">M↓</div>
      <div>
        <h1>mdview</h1>
        <p>轻量 Markdown 本地阅读与编辑 · Rust + WebView2</p>
      </div>
    </div>
    <div id="welcome-error" class="error-panel" hidden></div>
    <p class="welcome-lead">双击 .md 文件，或者现在打开一个。阅读、目录、代码高亮、本地图片、深色模式、F2/F3 编辑和导出都在本机完成，文件不会离开电脑。</p>
    <div class="welcome-actions">
      <button id="open-main" class="btn primary">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/></svg>
        打开 Markdown 文件
      </button>
      <button id="open-secondary" class="btn">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><rect x="2.5" y="6" width="19" height="12" rx="2"/><path d="M6 10h.01M9.5 10h.01M13 10h.01M16.5 10h.01M7.5 17h9" stroke-linecap="round"/></svg>
        快捷键
      </button>
    </div>
    <div class="drop-zone">
      <strong>也可以把文件拖到这里</strong>
      支持 .md · .markdown · .mdown · .mkd · .txt
    </div>
    <div class="welcome-section-title">
      <span>最近打开</span>
      <button id="clear-recents" class="text-btn">清除记录</button>
    </div>
    <div id="recent-list" class="recent-list"></div>
    <div class="welcome-foot">
      <span><kbd class="key">Ctrl</kbd> + <kbd class="key">N</kbd> 新窗口</span>
      <span><kbd class="key">F2</kbd> 就地编辑</span>
      <span><kbd class="key">F3</kbd> 源码分屏</span>
      <span><kbd class="key">Ctrl</kbd> + <kbd class="key">P</kbd> 导出 PDF</span>
      <span><kbd class="key">F1</kbd> 全部快捷键</span>
    </div>
  </div>
</section>

<main id="layout">
  <aside id="toc-panel">
    <div class="toc-head">
      <span>文档目录 <em id="toc-count"></em></span>
      <button id="toc-close" class="text-btn" title="隐藏目录">隐藏</button>
    </div>
    <nav id="toc" aria-label="文档目录"></nav>
    <div class="toc-progress">
      <div class="toc-progress-label"><span>阅读进度</span><span id="toc-percent">0%</span></div>
      <div class="toc-progress-track"><div class="toc-progress-bar" id="toc-progress-bar"></div></div>
    </div>
  </aside>
  <section id="reader">
    <div class="article-card">
      <article id="content" class="markdown-body" aria-live="polite"></article>
    </div>
  </section>
</main>

<div id="wysiwyg-bar" class="edit-bar" hidden>
  <div class="edit-label"><span class="edit-key">F2</span><span>就地编辑</span><span id="wysiwyg-status" class="edit-status">已保存</span></div>
  <div class="edit-actions">
    <button id="wysiwyg-save" class="edit-btn">保存</button>
    <button id="wysiwyg-finish" class="edit-btn primary">完成</button>
  </div>
</div>

<section id="split-editor" hidden>
  <div class="split-topbar">
    <div class="edit-label"><span class="edit-key">F3</span><span>源码分屏编辑</span><span id="split-status" class="edit-status">已保存</span></div>
    <div id="split-meta" class="split-meta"></div>
    <div class="edit-actions">
      <button id="split-save" class="edit-btn">保存</button>
      <button id="split-finish" class="edit-btn primary">退出编辑 Esc</button>
    </div>
  </div>
  <div class="split-main">
    <section class="split-pane source-pane">
      <div class="split-pane-head"><span>Markdown 源码</span><span id="source-meta"></span></div>
      <textarea id="source-editor" spellcheck="false" wrap="off" aria-label="Markdown 源码"></textarea>
    </section>
    <section class="split-pane preview-pane">
      <div class="split-pane-head"><span>实时预览</span><span>只读</span></div>
      <div id="preview-scroll" class="preview-scroll">
        <article id="preview-content" class="markdown-body"></article>
      </div>
    </section>
  </div>
</section>

<div class="statusbar">
  <span id="status-file">未打开文件</span>
  <span class="status-sep"></span>
  <span id="status-meta"></span>
</div>

<button id="backtop" title="回到顶部" aria-label="回到顶部">
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m6 15 6-6 6 6" stroke-linecap="round" stroke-linejoin="round"/></svg>
</button>

<div id="context-menu" role="menu">
  <div class="menu-label">窗口</div>
  <button class="menu-item" data-action="new-window"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M12 5v14M5 12h14" stroke-linecap="round"/></svg></span>新建窗口<span class="shortcut">Ctrl N</span></button>
  <button class="menu-item" data-action="open-new"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/><path d="M12 11v5M9.5 13.5h5" stroke-linecap="round"/></svg></span>在新窗口打开<span class="shortcut">Ctrl Shift O</span></button>
  <div class="menu-divider"></div>
  <div class="menu-label">文档</div>
  <button class="menu-item" data-action="open"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/></svg></span>打开文件<span class="shortcut">Ctrl O</span></button>
  <button class="menu-item" data-action="reload"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M20 11a8 8 0 1 0-2.34 5.66M20 5v6h-6" stroke-linecap="round" stroke-linejoin="round"/></svg></span>重新加载<span class="shortcut">Ctrl R</span></button>
  <button class="menu-item" data-action="edit-wysiwyg"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m4 20 4.5-1 10-10a2.1 2.1 0 0 0-3-3l-10 10L4 20Z" stroke-linejoin="round"/><path d="m13.5 7.5 3 3" stroke-linecap="round"/></svg></span>就地编辑<span class="shortcut">F2</span></button>
  <button class="menu-item" data-action="edit-split"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M12 4v16"/></svg></span>源码分屏编辑<span class="shortcut">F3</span></button>
  <button class="menu-item" data-action="toc"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" stroke-linecap="round"/></svg></span>显示/隐藏目录<span class="shortcut">Ctrl B</span></button>
  <div class="menu-divider"></div>
  <div class="menu-label">外观</div>
  <div class="menu-row" id="theme-chips">
    <button class="menu-chip" data-action="theme" data-value="system">跟随系统</button>
    <button class="menu-chip" data-action="theme" data-value="light">浅色</button>
    <button class="menu-chip" data-action="theme" data-value="dark">深色</button>
  </div>
  <div class="accent-grid" id="accent-dots">
    <button class="accent-dot" data-action="accent" data-value="github" style="background:#0969da" title="GitHub"></button>
    <button class="accent-dot" data-action="accent" data-value="xhs" style="background:#e8263f" title="小红书"></button>
    <button class="accent-dot" data-action="accent" data-value="tech" style="background:#1677ff" title="科技"></button>
    <button class="accent-dot" data-action="accent" data-value="girl" style="background:#d6409f" title="少女"></button>
    <button class="accent-dot" data-action="accent" data-value="colorful" style="background:#7c3aed" title="多彩"></button>
    <button class="accent-dot" data-action="accent" data-value="duolingo" style="background:#3f9f1c" title="Duolingo"></button>
    <button class="accent-dot" data-action="accent" data-value="linear" style="background:#5e6ad2" title="Linear"></button>
    <button class="accent-dot" data-action="accent" data-value="notion" style="background:#b45309" title="Notion"></button>
  </div>
  <div class="menu-row">
    <button class="menu-chip" data-action="font-minus">A−</button>
    <button class="menu-chip" data-action="font-reset">A</button>
    <button class="menu-chip" data-action="font-plus">A+</button>
  </div>
  <div class="menu-divider"></div>
  <button class="menu-item" data-action="export"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M12 3v12m0 0 4-4m-4 4-4-4M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" stroke-linecap="round" stroke-linejoin="round"/></svg></span>导出单文件 HTML<span class="shortcut">Ctrl E</span></button>
  <button class="menu-item" data-action="export-pdf"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M6 2h8l4 4v16H6z" stroke-linejoin="round"/><path d="M14 2v5h5M9 13h6M9 17h4" stroke-linecap="round"/></svg></span>导出 PDF（无对话框）<span class="shortcut">Ctrl P</span></button>
  <button class="menu-item" data-action="print"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M7 9V4h10v5M7 18H5a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2M7 14h10v6H7z" stroke-linejoin="round"/></svg></span>打印…<span class="shortcut">Ctrl Shift P</span></button>
  <button class="menu-item" data-action="reveal"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/></svg></span>在资源管理器中显示</button>
  <button class="menu-item" data-action="copy-path"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><rect x="8" y="8" width="11" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h2" stroke-linejoin="round"/></svg></span>复制文件路径</button>
  <div class="menu-divider"></div>
  <button class="menu-item" data-action="settings"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="3.2"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.83 2.83-.06-.06a1.7 1.7 0 0 0-1.88-.34 1.7 1.7 0 0 0-1.03 1.56V21h-4v-.08A1.7 1.7 0 0 0 8.96 19.4a1.7 1.7 0 0 0-1.88.34l-.06.06-2.83-2.83.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.56-1.03H3v-4h.08A1.7 1.7 0 0 0 4.6 8.94a1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.83-2.83.06.06A1.7 1.7 0 0 0 8.96 4.6 1.7 1.7 0 0 0 10 3.04V3h4v.08a1.7 1.7 0 0 0 1.03 1.56 1.7 1.7 0 0 0 1.88-.34l.06-.06 2.83 2.83-.06.06A1.7 1.7 0 0 0 19.4 9c.13.62.66 1.08 1.3 1.08H21v4h-.3c-.64 0-1.17.46-1.3 1.08Z" stroke-linejoin="round"/></svg></span>阅读设置<span class="shortcut">Ctrl ,</span></button>
  <button class="menu-item" data-action="shortcuts"><span class="menu-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><rect x="2.5" y="6" width="19" height="12" rx="2"/><path d="M6 10h.01M9.5 10h.01M13 10h.01M16.5 10h.01M7.5 17h9" stroke-linecap="round"/></svg></span>键盘快捷键<span class="shortcut">F1</span></button>
</div>

<div id="drawer-mask"></div>
<aside id="drawer" aria-label="阅读设置">
  <div class="drawer-head">
    <h2>阅读设置</h2>
    <button id="drawer-close" class="icon-btn" aria-label="关闭设置">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m6 6 12 12M18 6 6 18" stroke-linecap="round"/></svg>
    </button>
  </div>
  <div class="setting-group">
    <div class="setting-label">主题</div>
    <div class="segmented">
      <button data-setting-theme="system">跟随系统</button>
      <button data-setting-theme="light">浅色</button>
      <button data-setting-theme="dark">深色</button>
    </div>
  </div>
  <div class="setting-group">
    <div class="setting-label">配色风格</div>
    <div class="swatches">
      <button class="swatch" data-accent="github" style="--swatch:#0969da"><i></i><span>GitHub</span></button>
      <button class="swatch" data-accent="xhs" style="--swatch:#e8263f"><i></i><span>小红书</span></button>
      <button class="swatch" data-accent="tech" style="--swatch:#1677ff"><i></i><span>科技</span></button>
      <button class="swatch" data-accent="girl" style="--swatch:#d6409f"><i></i><span>少女</span></button>
      <button class="swatch" data-accent="colorful" style="--swatch:#7c3aed"><i></i><span>多彩</span></button>
      <button class="swatch" data-accent="duolingo" style="--swatch:#3f9f1c"><i></i><span>Duolingo</span></button>
      <button class="swatch" data-accent="linear" style="--swatch:#5e6ad2"><i></i><span>Linear</span></button>
      <button class="swatch" data-accent="notion" style="--swatch:#b45309"><i></i><span>Notion</span></button>
    </div>
  </div>
  <div class="setting-group">
    <div class="setting-label">字号 <span id="font-value">16px</span></div>
    <div class="size-row">
      <input id="font-range" type="range" min="13" max="24" step="1" value="16" aria-label="正文字号">
    </div>
  </div>
  <div class="setting-group">
    <div class="setting-label">版心宽度</div>
    <div class="segmented">
      <button data-setting-width="660">窄</button>
      <button data-setting-width="760">标准</button>
      <button data-setting-width="900">宽</button>
    </div>
  </div>
  <div class="setting-group">
    <div class="setting-label">正文字体</div>
    <div class="segmented">
      <button data-setting-font="sans">无衬线</button>
      <button data-setting-font="serif">衬线</button>
    </div>
  </div>
  <div class="setting-group">
    <div class="toggle-row"><span>显示文档目录</span><label class="switch"><input id="toc-toggle" type="checkbox" checked><span></span></label></div>
  </div>
  <div class="setting-group">
    <div class="setting-label">系统集成</div>
    <div class="welcome-actions">
      <button id="register-btn" class="btn">关联 .md 文件</button>
      <button id="unregister-btn" class="btn">解除关联</button>
    </div>
    <div style="margin-top:8px"><button id="config-btn" class="text-btn">打开设置文件所在目录</button></div>
  </div>
  <div class="about-box">
    <strong>mdview __VERSION__</strong><br>
    Rust + Windows WebView2 本地 Markdown 阅读器。渲染在当前进程内完成，不上传文件。<br>
    WebView2：__WEBVIEW__<br>
    <div class="about-links">
      <button id="website-btn">功能参考 mdview.cn</button>
      <button id="drawer-shortcuts">快捷键</button>
    </div>
  </div>
</aside>

<div id="modal-mask">
  <div class="modal">
    <h2>键盘快捷键</h2>
    <table class="shortcut-table">
      <tr><td>新建窗口</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">N</kbd></td></tr>
      <tr><td>在新窗口打开</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">Shift</kbd> + <kbd class="key">O</kbd></td></tr>
      <tr><td>打开文件</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">O</kbd></td></tr>
      <tr><td>重新加载</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">R</kbd></td></tr>
      <tr><td>F2 就地编辑 / 完成</td><td><kbd class="key">F2</kbd></td></tr>
      <tr><td>F3 源码分屏 / 完成</td><td><kbd class="key">F3</kbd></td></tr>
      <tr><td>编辑中保存</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">S</kbd></td></tr>
      <tr><td>显示 / 隐藏目录</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">B</kbd></td></tr>
      <tr><td>切换主题</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">D</kbd></td></tr>
      <tr><td>放大 / 缩小字号</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">+</kbd> / <kbd class="key">−</kbd></td></tr>
      <tr><td>导出单文件 HTML</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">E</kbd></td></tr>
      <tr><td>导出 PDF（无对话框）</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">P</kbd></td></tr>
      <tr><td>打开系统打印</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">Shift</kbd> + <kbd class="key">P</kbd></td></tr>
      <tr><td>阅读设置</td><td><kbd class="key">Ctrl</kbd> + <kbd class="key">,</kbd></td></tr>
      <tr><td>关闭窗口</td><td><kbd class="key">Esc</kbd> / <kbd class="key">Ctrl</kbd> + <kbd class="key">W</kbd></td></tr>
    </table>
    <div class="modal-actions"><button id="modal-close" class="btn primary">知道了</button></div>
  </div>
</div>

<div id="lightbox"><span class="close-hint">点击空白处或按 Esc 关闭</span><img id="lightbox-img" alt=""></div>
<div id="toast" role="status" aria-live="polite"></div>

<script>window.__BOOTSTRAP__ = __BOOTSTRAP_JSON__;</script>
<script>
"##,
    );
    html.push_str(HIGHLIGHT_JS);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(KATEX_JS);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(AUTO_RENDER_JS);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(MERMAID_JS);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(TURNDOWN_JS);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(TURNDOWN_GFM_JS);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(APP_JS);
    html.push_str("\n</script>\n</body>\n</html>\n");
    html = html
        .replace("__BOOTSTRAP_JSON__", &bootstrap)
        .replace("__VERSION__", version)
        .replace(
            "__WEBVIEW__",
            &crate::document::html_escape(webview_version),
        );
    html
}
