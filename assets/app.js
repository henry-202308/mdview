(() => {
  "use strict";

  const boot = window.__BOOTSTRAP__ || {};
  const defaults = {
    theme: "system",
    accent: "github",
    font_size: 16,
    reading_width: 760,
    toc: true,
    font: "sans",
    recent: [],
  };
  const ACCENTS = [
    ["github", "GitHub", "#0969da"],
    ["xhs", "小红书", "#e8263f"],
    ["tech", "科技", "#1677ff"],
    ["girl", "少女", "#d6409f"],
    ["colorful", "多彩", "#7c3aed"],
    ["duolingo", "Duolingo", "#3f9f1c"],
    ["linear", "Linear", "#5e6ad2"],
    ["notion", "Notion", "#b45309"],
  ];
  const BLOCKED_TAGS = new Set([
    "script", "style", "link", "meta", "base", "iframe", "frame", "frameset",
    "object", "embed", "applet", "form", "button", "textarea", "select",
    "video", "audio", "source", "track", "canvas", "svg", "math", "portal",
  ]);
  const state = {
    doc: null,
    settings: Object.assign({}, defaults, boot.settings || {}),
    resolvedTheme: "light",
    tocLinks: [],
    activeId: "",
    mermaidSeq: 0,
    renderToken: 0,
    previewToken: 0,
    saveTimer: 0,
    hideTimer: 0,
    edit: null,
    previewTimer: 0,
    splitSyncing: false,
  };
  if (!Array.isArray(state.settings.recent)) state.settings.recent = [];
  if (!state.settings.recent.length && Array.isArray(boot.recents)) {
    state.settings.recent = boot.recents.filter((item) => typeof item === "string");
  }

  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];
  const article = $("#content");
  const toc = $("#toc");
  const body = document.body;
  const root = document.documentElement;
  const mqDark = window.matchMedia("(prefers-color-scheme: dark)");

  const icons = {
    toc: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" stroke-linecap="round"/></svg>',
    open: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/></svg>',
    sun: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.42 1.42M17.65 17.65l1.42 1.42M2 12h2M20 12h2M4.93 19.07l1.42-1.42M17.65 6.35l1.42-1.42" stroke-linecap="round"/></svg>',
    moon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5 8.5 8.5 0 1 0 20.5 14.5Z" stroke-linejoin="round"/></svg>',
    settings: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.83 2.83-.06-.06a1.7 1.7 0 0 0-1.88-.34 1.7 1.7 0 0 0-1.03 1.56V21h-4v-.08A1.7 1.7 0 0 0 8.96 19.4a1.7 1.7 0 0 0-1.88.34l-.06.06-2.83-2.83.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.56-1.03H3v-4h.08A1.7 1.7 0 0 0 4.6 8.94a1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.83-2.83.06.06A1.7 1.7 0 0 0 8.96 4.6 1.7 1.7 0 0 0 10 3.04V3h4v.08a1.7 1.7 0 0 0 1.03 1.56 1.7 1.7 0 0 0 1.88-.34l.06-.06 2.83 2.83-.06.06A1.7 1.7 0 0 0 19.4 9c.13.62.66 1.08 1.3 1.08H21v4h-.3c-.64 0-1.17.46-1.3 1.08Z" stroke-linejoin="round"/></svg>',
    folder: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2h7.5A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5v-10Z" stroke-linejoin="round"/></svg>',
    download: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M12 3v12m0 0 4-4m-4 4-4-4M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" stroke-linecap="round" stroke-linejoin="round"/></svg>',
    print: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M7 9V4h10v5M7 18H5a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2M7 14h10v6H7z" stroke-linejoin="round"/></svg>',
    refresh: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M20 11a8 8 0 1 0-2.34 5.66M20 5v6h-6" stroke-linecap="round" stroke-linejoin="round"/></svg>',
    external: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M14 4h6v6M20 4l-9 9M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5" stroke-linecap="round" stroke-linejoin="round"/></svg>',
    info: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01" stroke-linecap="round"/></svg>',
    copy: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><rect x="8" y="8" width="11" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h2" stroke-linejoin="round"/></svg>',
    close: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m6 6 12 12M18 6 6 18" stroke-linecap="round"/></svg>',
    keyboard: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><rect x="2.5" y="6" width="19" height="12" rx="2"/><path d="M6 10h.01M9.5 10h.01M13 10h.01M16.5 10h.01M6 13.5h.01M9.5 13.5h.01M13 13.5h.01M16.5 13.5h.01M7.5 17h9" stroke-linecap="round"/></svg>',
    up: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m6 15 6-6 6 6" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  };

  function icon(name) {
    return `<span class="menu-icon">${icons[name] || ""}</span>`;
  }

  function ipc(cmd, extra = {}) {
    try {
      window.ipc.postMessage(JSON.stringify(Object.assign({ cmd }, extra)));
    } catch (err) {
      console.error("IPC failed", err);
    }
  }

  function ready(callback) {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", callback, { once: true });
    } else {
      callback();
    }
  }

  function resolveTheme() {
    if (state.settings.theme === "dark") return "dark";
    if (state.settings.theme === "light") return "light";
    return mqDark.matches ? "dark" : "light";
  }

  function applySettings(options = {}) {
    const settings = state.settings;
    const previousTheme = state.resolvedTheme;
    state.resolvedTheme = resolveTheme();
    root.dataset.theme = state.resolvedTheme;
    root.dataset.accent = settings.accent;
    root.dataset.font = settings.font;
    root.style.setProperty("--font-size", `${settings.font_size}px`);
    root.style.setProperty("--reading-width", `${settings.reading_width}px`);
    body.classList.toggle("toc-closed", !settings.toc);
    $("#btn-toc")?.classList.toggle("active", settings.toc);
    updateThemeButton();
    updateSettingsUI();
    const meta = $('meta[name="theme-color"]');
    if (meta) meta.content = state.resolvedTheme === "dark" ? "#0d1117" : "#f5f6f7";
    if (!options.silentTheme && previousTheme !== state.resolvedTheme && state.doc && !state.edit) {
      renderMermaid();
    }
  }

  function updateThemeButton() {
    const btn = $("#btn-theme");
    if (!btn) return;
    const dark = state.resolvedTheme === "dark";
    btn.innerHTML = dark ? icons.sun : icons.moon;
    btn.title = dark ? "切换浅色主题 (Ctrl+D)" : "切换深色主题 (Ctrl+D)";
  }

  function commitSettings(patch, options = {}) {
    Object.assign(state.settings, patch);
    applySettings({ silentTheme: options.silentTheme });
    scheduleSave();
  }

  function scheduleSave() {
    clearTimeout(state.saveTimer);
    state.saveTimer = setTimeout(() => {
      ipc("saveSettings", { settings: state.settings });
    }, 220);
  }

  function updateSettingsUI() {
    $$("[data-setting-theme]").forEach((button) => {
      button.classList.toggle("active", button.dataset.settingTheme === state.settings.theme);
    });
    $$("[data-setting-width]").forEach((button) => {
      button.classList.toggle("active", Number(button.dataset.settingWidth) === state.settings.reading_width);
    });
    $$("[data-setting-font]").forEach((button) => {
      button.classList.toggle("active", button.dataset.settingFont === state.settings.font);
    });
    $$(".swatch[data-accent]").forEach((button) => {
      button.classList.toggle("active", button.dataset.accent === state.settings.accent);
    });
    const range = $("#font-range");
    if (range) range.value = String(state.settings.font_size);
    const value = $("#font-value");
    if (value) value.textContent = `${state.settings.font_size}px`;
    const toggle = $("#toc-toggle");
    if (toggle) toggle.checked = state.settings.toc;
    $$("#theme-chips [data-action='theme']").forEach((button) => {
      button.classList.toggle("active", button.dataset.value === state.settings.theme);
    });
    $$("#accent-dots .accent-dot").forEach((button) => {
      button.classList.toggle("active", button.dataset.accent === state.settings.accent);
    });
  }

  function toggleTheme() {
    const next = state.resolvedTheme === "dark" ? "light" : "dark";
    commitSettings({ theme: next });
    toast(next === "dark" ? "已切换到深色主题" : "已切换到浅色主题");
  }

  function toggleToc(force) {
    const next = typeof force === "boolean" ? force : !state.settings.toc;
    commitSettings({ toc: next }, { silentTheme: true });
    $("#btn-toc")?.classList.toggle("active", next);
  }

  function showChrome(sticky = false) {
    body.classList.add("chrome-visible");
    clearTimeout(state.hideTimer);
    if (!sticky) {
      state.hideTimer = setTimeout(() => {
        if (!body.classList.contains("reading")) return;
        if ($("#drawer")?.classList.contains("open")) return;
        if ($("#context-menu")?.classList.contains("open")) return;
        body.classList.remove("chrome-visible");
      }, 2300);
    }
  }

  function scheduleChromeHide() {
    if (!body.classList.contains("reading")) return;
    clearTimeout(state.hideTimer);
    state.hideTimer = setTimeout(() => {
      if ($("#drawer")?.classList.contains("open")) return;
      if ($("#context-menu")?.classList.contains("open")) return;
      body.classList.remove("chrome-visible");
    }, 1500);
  }

  function toast(message, type = "") {
    const node = $("#toast");
    if (!node) return;
    node.textContent = message;
    node.className = type === "error" ? "show error" : "show";
    clearTimeout(state.toastTimer);
    state.toastTimer = setTimeout(() => {
      node.className = type === "error" ? "error" : "";
    }, 2600);
  }

  function showError(message) {
    toast(message, "error");
    if (!state.doc) {
      const panel = $("#welcome-error");
      if (panel) {
        panel.hidden = false;
        panel.textContent = message;
      }
    }
  }

  function setEditStatus(text, kind = "") {
    if (!state.edit) return;
    const node = state.edit.mode === "split" ? $("#split-status") : $("#wysiwyg-status");
    if (!node) return;
    node.textContent = text;
    node.className = `edit-status ${kind}`.trim();
  }

  function clearEditTimers() {
    if (!state.edit) return;
    clearTimeout(state.edit.saveTimer);
    clearTimeout(state.edit.previewTimer);
  }

  function teardownEditUi() {
    clearEditTimers();
    const mode = state.edit?.mode;
    if (mode === "wysiwyg") {
      article.contentEditable = "false";
      article.removeAttribute("contenteditable");
      article.spellcheck = false;
      body.classList.remove("wysiwyg-mode");
      $("#wysiwyg-bar").hidden = true;
    } else if (mode === "split") {
      body.classList.remove("split-mode");
      $("#split-editor").hidden = true;
      $("#preview-content").replaceChildren();
    }
    state.edit = null;
  }

  function enterWysiwyg() {
    if (!state.doc) {
      toast("请先打开一个 Markdown 文件", true);
      return;
    }
    if (state.edit) {
      toast("已经在编辑模式中，按 Esc 退出", true);
      return;
    }
    if (typeof TurndownService !== "function") {
      toast("编辑器资源加载失败", true);
      return;
    }
    closeDrawer();
    closeModal();
    hideContextMenu();
    closeLightbox();
    state.edit = { mode: "wysiwyg", dirty: false, saving: false, pendingFinish: false, saveTimer: 0, revision: 0, inFlight: 0 };
    ipc("setEditing", { editing: true });
    $$(".heading-anchor", article).forEach((node) => node.remove());
    $$(".code-head", article).forEach((node) => node.remove());
    $$(".mermaid-box, math, .katex-display, img", article).forEach((node) => {
      node.setAttribute("contenteditable", "false");
    });
    article.contentEditable = "true";
    article.spellcheck = false;
    body.classList.add("wysiwyg-mode");
    $("#wysiwyg-bar").hidden = false;
    showChrome(true);
    setEditStatus("已保存", "saved");
    article.focus();
    const selection = window.getSelection();
    if (selection) {
      const range = document.createRange();
      range.selectNodeContents(article);
      range.collapse(true);
      selection.removeAllRanges();
      selection.addRange(range);
    }
  }

  function enterSplit() {
    if (!state.doc) {
      toast("请先打开一个 Markdown 文件", true);
      return;
    }
    if (state.edit) {
      toast("已经在编辑模式中，按 Esc 退出", true);
      return;
    }
    closeDrawer();
    closeModal();
    hideContextMenu();
    closeLightbox();
    state.edit = { mode: "split", dirty: false, saving: false, pendingFinish: false, saveTimer: 0, previewTimer: 0, revision: 0, inFlight: 0 };
    ipc("setEditing", { editing: true });
    body.classList.add("split-mode");
    $("#split-editor").hidden = false;
    const source = $("#source-editor");
    source.value = state.doc.raw || "";
    renderPreviewHtml(state.doc.html || "");
    updateSplitMeta();
    setEditStatus("已保存", "saved");
    showChrome(true);
    source.focus();
    source.setSelectionRange(0, 0);
    source.scrollTop = 0;
    $("#preview-scroll").scrollTop = 0;
    requestAnimationFrame(() => {
      source.scrollTop = 0;
      $("#preview-scroll").scrollTop = 0;
    });
  }

  function finishEdit() {
    if (!state.edit) return;
    clearEditTimers();
    if (state.edit.saving) {
      state.edit.pendingFinish = true;
      return;
    }
    if (!state.edit.dirty) {
      state.edit.pendingFinish = false;
      ipc("finishEdit");
      return;
    }
    state.edit.pendingFinish = true;
    performEditSave(true);
  }

  function scheduleAutoSave() {
    if (!state.edit) return;
    state.edit.revision += 1;
    state.edit.dirty = true;
    setEditStatus("未保存", "dirty");
    clearTimeout(state.edit.saveTimer);
    state.edit.saveTimer = setTimeout(() => performEditSave(false), 900);
  }

  function performEditSave(finish) {
    if (!state.edit) return;
    if (state.edit.saving) {
      if (finish) state.edit.pendingFinish = true;
      return;
    }
    const mode = state.edit.mode;
    const markdown = mode === "wysiwyg" ? articleToMarkdown() : $("#source-editor").value;
    if (mode === "wysiwyg" && !markdown.trim()) {
      toast("文档为空，未保存", true);
      setEditStatus("未保存", "dirty");
      return;
    }
    state.edit.saving = true;
    state.edit.inFlight = state.edit.revision;
    setEditStatus("保存中…", "saving");
    ipc("saveEdit", {
      markdown,
      mode: mode === "wysiwyg" ? "body" : "full",
      finish: Boolean(finish),
    });
  }

  function onEditSaved(payload) {
    if (!state.edit) return;
    state.edit.saving = false;
    const current = state.edit.revision;
    const saved = state.edit.inFlight;
    const hasNewChanges = current > saved;
    state.edit.dirty = hasNewChanges;
    if (payload && state.doc) {
      state.doc.word_count = payload.wordCount ?? state.doc.word_count;
      state.doc.char_count = payload.charCount ?? state.doc.char_count;
      state.doc.read_minutes = payload.readMinutes ?? state.doc.read_minutes;
      state.doc.modified_ms = payload.modifiedMs ?? state.doc.modified_ms;
      if (payload.raw) state.doc.raw = payload.raw;
      if (state.edit?.mode === "split") {
        state.edit.previewWords = payload.wordCount ?? state.edit.previewWords;
        updateSplitMeta();
      }
      if (payload.title) {
        state.doc.title = payload.title;
        $("#doc-title").textContent = payload.title;
        document.title = `${payload.title} — mdview`;
      }
      updateStatus();
    }
    setEditStatus(hasNewChanges ? "未保存" : "已保存", hasNewChanges ? "dirty" : "saved");
    if (hasNewChanges) {
      clearTimeout(state.edit.saveTimer);
      state.edit.saveTimer = setTimeout(() => performEditSave(false), 900);
    }
    if (state.edit?.pendingFinish) {
      state.edit.pendingFinish = false;
      if (state.edit.dirty) {
        performEditSave(true);
      } else {
        ipc("finishEdit");
      }
    }
  }

  function onEditError(message) {
    if (!state.edit) {
      toast(message, true);
      return;
    }
    state.edit.saving = false;
    state.edit.dirty = true;
    setEditStatus("保存失败", "dirty");
    toast(message, true);
  }

  function articleToMarkdown() {
    const clone = article.cloneNode(true);
    const placeholders = [];
    const mathNodes = [...clone.querySelectorAll("math")];
    for (const math of mathNodes) {
      const wrapper = math.closest(".katex") || math;
      if (!wrapper || wrapper.closest(".mermaid-box")) continue;
      const annotation = math.querySelector('annotation[encoding="application/x-tex"]');
      const tex = (annotation?.textContent || "").trim();
      if (!tex) continue;
      const display = math.getAttribute("display") === "block" || Boolean(wrapper.closest(".katex-display"));
      const token = `MDVIEWMATHTOKEN${placeholders.length}X`;
      const replacement = document.createElement("span");
      replacement.textContent = token;
      wrapper.replaceWith(replacement);
      placeholders.push({ token, value: display ? `\n\n$$\n${tex}\n$$\n\n` : `$${tex}$` });
    }
    const service = new TurndownService({
      headingStyle: "atx",
      hr: "---",
      bulletListMarker: "-",
      codeBlockStyle: "fenced",
      emDelimiter: "*",
      strongDelimiter: "**",
    });
    if (typeof turndownPluginGfm !== "undefined" && turndownPluginGfm.gfm) {
      service.use(turndownPluginGfm.gfm);
    }
    service.remove(["script", "style"]);
    service.addRule("mdImage", {
      filter: "img",
      replacement: (_content, node) => {
        const src = node.getAttribute("src") || "";
        const alt = node.getAttribute("alt") || "";
        if (!src) return alt;
        if (src.startsWith("data:")) return alt;
        const source = markdownImageSource(src);
        if (!source) return alt;
        const title = node.getAttribute("title");
        return `![${alt}](${source}${title ? ` "${title.replace(/"/g, '\\"')}"` : ""})`;
      },
    });
    service.addRule("mermaidBlock", {
      filter: (node) => node.classList?.contains("mermaid-box"),
      replacement: (_content, node) => `\n\n\`\`\`mermaid\n${node.dataset.code || ""}\n\`\`\`\n\n`,
    });
    service.addRule("mathBlock", {
      filter: (node) => node.nodeName.toLowerCase() === "math",
      replacement: (content, node) => {
        const annotation = node.querySelector('annotation[encoding="application/x-tex"]');
        if (!annotation) return content;
        const tex = (annotation.textContent || "").trim();
        if (!tex) return content;
        const display = node.getAttribute("display") === "block" || Boolean(node.closest(".katex-display"));
        return display ? `\n\n$$\n${tex}\n$$\n\n` : `$${tex}$`;
      },
    });
    let markdown = service.turndown(clone).trim();
    for (const item of placeholders) {
      markdown = markdown.split(item.token).join(item.value);
    }
    return markdown.trim();
  }

  function markdownImageSource(src) {
    if (src.startsWith("/local?") || src.includes("/local?path=")) {
      try {
        const url = new URL(src, location.href);
        const target = url.searchParams.get("path");
        if (target && state.doc) return encodeMarkdownPath(relativeDocPath(state.doc.dir, target));
      } catch {}
    }
    if (src.startsWith("file://")) return src;
    if (/^https?:/i.test(src)) return src;
    return src;
  }

  function relativeDocPath(fromDir, target) {
    const from = String(fromDir || "").replace(/\//g, "\\").split("\\").filter(Boolean);
    const to = String(target || "").replace(/\//g, "\\").split("\\").filter(Boolean);
    if (!from.length || !to.length) return target.replace(/\\/g, "/");
    if (from[0].toLowerCase() !== to[0].toLowerCase()) {
      return `file:///${to.join("/")}`;
    }
    let index = 0;
    while (index < from.length && index < to.length - 1 && from[index].toLowerCase() === to[index].toLowerCase()) {
      index += 1;
    }
    return [...Array(Math.max(0, from.length - index)).fill(".."), ...to.slice(index)].join("/");
  }

  function encodeMarkdownPath(value) {
    if (value.startsWith("file:///")) return value;
    return value
      .split("/")
      .map((part) => encodeURIComponent(part).replace(/%2E/gi, "."))
      .join("/")
      .replace(/%20/g, "%20");
  }

  function enhancePreview(rootNode) {
    $$("code.language-mermaid", rootNode).forEach((code) => {
      const pre = code.closest("pre");
      if (!pre) return;
      const box = document.createElement("div");
      box.className = "mermaid-box";
      box.dataset.code = code.textContent;
      const diagram = document.createElement("div");
      diagram.className = "mermaid-diagram";
      box.append(diagram);
      pre.replaceWith(box);
    });
    $$("pre > code", rootNode).forEach((code) => {
      if (code.closest(".mermaid-box")) return;
      if (!window.hljs || code.textContent.length > 200000) return;
      const lang = (code.className.match(/language-([\w+#.-]+)/i) || [])[1];
      try {
        if (lang && hljs.getLanguage(lang)) hljs.highlightElement(code);
      } catch {}
    });
    renderMath(rootNode);
    renderMermaid(rootNode, "previewToken");
  }

  function renderPreviewHtml(html) {
    const preview = $("#preview-content");
    if (!preview) return;
    const template = document.createElement("template");
    template.innerHTML = html || "";
    sanitize(template.content);
    preview.replaceChildren(template.content);
    enhancePreview(preview);
  }

  function updateSplitMeta() {
    if (!state.edit || state.edit.mode !== "split") return;
    const source = $("#source-editor").value;
    const lines = source ? source.split("\n").length : 0;
    const chars = source.length;
    const words = state.edit.previewWords ?? state.doc?.word_count ?? 0;
    $("#source-meta").textContent = `${lines} 行 · ${chars} 字符`;
    $("#split-meta").textContent = `${state.doc?.file_name || ""} · ${words} 字 · 自动保存`;
  }

  function schedulePreview() {
    if (!state.edit || state.edit.mode !== "split") return;
    state.edit.revision += 1;
    state.edit.dirty = true;
    setEditStatus("未保存", "dirty");
    updateSplitMeta();
    clearTimeout(state.edit.previewTimer);
    state.edit.previewTimer = setTimeout(() => {
      if (!state.edit || state.edit.mode !== "split") return;
      ipc("renderPreview", { source: $("#source-editor").value });
    }, 200);
    clearTimeout(state.edit.saveTimer);
    state.edit.saveTimer = setTimeout(() => performEditSave(false), 1000);
  }

  function onPreview(payload) {
    if (!state.edit || state.edit.mode !== "split") return;
    state.edit.previewWords = payload.wordCount ?? 0;
    renderPreviewHtml(payload.html || "");
    updateSplitMeta();
    if (!state.edit.dirty) setEditStatus("已保存", "saved");
  }

  function syncSplitScroll(source, target) {
    if (state.splitSyncing) return;
    const sourceMax = source.scrollHeight - source.clientHeight;
    const targetMax = target.scrollHeight - target.clientHeight;
    if (sourceMax <= 0 || targetMax <= 0) return;
    state.splitSyncing = true;
    target.scrollTop = (source.scrollTop / sourceMax) * targetMax;
    requestAnimationFrame(() => { state.splitSyncing = false; });
  }

  function renderRecent() {
    const list = $("#recent-list");
    if (!list) return;
    const recent = state.settings.recent || [];
    list.replaceChildren();
    if (!recent.length) {
      const empty = document.createElement("div");
      empty.className = "recent-empty";
      empty.textContent = "还没有最近打开的文件";
      list.append(empty);
      return;
    }
    for (const path of recent) {
      const name = path.split(/[\\/]/).pop() || path;
      const dir = path.slice(0, Math.max(0, path.length - name.length - 1));
      const button = document.createElement("button");
      button.type = "button";
      button.className = "recent-item";
      button.title = path;
      button.dataset.path = path;
      const badge = document.createElement("span");
      badge.className = "recent-icon";
      badge.textContent = "MD";
      const main = document.createElement("span");
      main.className = "recent-main";
      const title = document.createElement("span");
      title.className = "recent-name";
      title.textContent = name;
      const sub = document.createElement("span");
      sub.className = "recent-path";
      sub.textContent = dir || path;
      main.append(title, sub);
      button.append(badge, main);
      button.addEventListener("click", () => ipc("openPath", { path }));
      list.append(button);
    }
  }

  function clearRecents() {
    commitSettings({ recent: [] });
    renderRecent();
    toast("已清除最近打开记录");
  }

  function recordRecent(path) {
    if (!path) return;
    const recent = state.settings.recent.filter((item) => item.toLowerCase() !== path.toLowerCase());
    recent.unshift(path);
    state.settings.recent = recent.slice(0, 10);
    renderRecent();
    scheduleSave();
  }

  function stripHtml(value) {
    const template = document.createElement("template");
    template.innerHTML = value;
    return (template.content.textContent || "").trim();
  }

  function slugify(value) {
    const base = stripHtml(value)
      .normalize("NFKC")
      .toLowerCase()
      .replace(/[`*_~#[\](){}<>]/g, "")
      .replace(/[^\p{L}\p{N}\s\-_]/gu, "")
      .trim()
      .replace(/\s+/g, "-");
    return base || "section";
  }

  function unsafeUrl(value) {
    const normalized = String(value || "").trim().replace(/[\u0000-\u0020]+/g, "").toLowerCase();
    return normalized.startsWith("javascript:")
      || normalized.startsWith("vbscript:")
      || normalized.startsWith("data:text/html");
  }

  function sanitize(rootNode) {
    const walker = document.createTreeWalker(rootNode, NodeFilter.SHOW_ELEMENT);
    let node = walker.nextNode();
    while (node) {
      const current = node;
      node = walker.nextNode();
      const tag = current.tagName.toLowerCase();
      if (BLOCKED_TAGS.has(tag)) {
        current.remove();
        continue;
      }
      for (const attr of [...current.attributes]) {
        const name = attr.name.toLowerCase();
        if (name.startsWith("on") || name === "style" || name === "srcdoc"
          || name === "srcset" || name === "ping" || name === "formaction") {
          current.removeAttribute(attr.name);
          continue;
        }
        if ((name === "href" || name === "src" || name === "action") && unsafeUrl(attr.value)) {
          current.removeAttribute(attr.name);
        }
      }
      if (tag === "a") {
        current.setAttribute("rel", "noopener noreferrer");
        if (current.getAttribute("href") && !current.getAttribute("href").startsWith("#")) {
          current.setAttribute("target", "_blank");
        }
      } else if (tag === "img") {
        current.setAttribute("loading", "lazy");
        current.setAttribute("decoding", "async");
        if (!current.getAttribute("alt")) current.setAttribute("alt", "");
      } else if (tag === "input") {
        if (current.getAttribute("type") !== "checkbox") {
          current.remove();
          continue;
        }
        current.setAttribute("disabled", "");
      }
    }
  }

  function uniqueHeadingId(text, used) {
    const base = slugify(text);
    let id = base;
    let index = 2;
    while (used.has(id) || document.getElementById(id)) {
      id = `${base}-${index++}`;
    }
    used.add(id);
    return id;
  }

  function decorateArticle() {
    const used = new Set();
    $$("h1, h2, h3, h4, h5, h6", article).forEach((heading) => {
      const text = heading.textContent.trim();
      const existing = heading.id && !used.has(heading.id) && !document.querySelector(`[data-mdview-id="${heading.id}"]`);
      const id = existing ? heading.id : uniqueHeadingId(text, used);
      used.add(id);
      heading.id = id;
      heading.dataset.mdviewId = id;
      const anchor = document.createElement("a");
      anchor.className = "heading-anchor";
      anchor.href = `#${encodeURIComponent(id)}`;
      anchor.textContent = "#";
      anchor.setAttribute("aria-label", `链接到 ${text}`);
      heading.prepend(anchor);
    });

    $$("table", article).forEach((table) => {
      if (table.parentElement?.classList.contains("table-wrap")) return;
      const wrap = document.createElement("div");
      wrap.className = "table-wrap";
      table.replaceWith(wrap);
      wrap.append(table);
    });

    $$("img", article).forEach((image) => {
      image.addEventListener("error", () => {
        image.classList.add("image-error");
        image.alt = image.alt || "图片加载失败";
      }, { once: true });
      const parent = image.parentElement;
      if (parent && parent.matches("p, li, td, th") && parent.textContent.trim() === "") {
        image.classList.add("inline-img");
      }
    });

    $$("code.language-mermaid", article).forEach((code) => {
      const pre = code.closest("pre");
      if (!pre) return;
      const box = document.createElement("div");
      box.className = "mermaid-box";
      box.dataset.code = code.textContent;
      const diagram = document.createElement("div");
      diagram.className = "mermaid-diagram";
      box.append(diagram);
      pre.replaceWith(box);
    });

    $$("pre > code", article).forEach((code) => {
      if (code.closest(".mermaid-box")) return;
      const pre = code.parentElement;
      const match = (code.className || "").match(/language-([\w+#.-]+)/i);
      const lang = match ? match[1] : "";
      if (window.hljs && code.textContent.length < 200000) {
        try {
          if (lang && hljs.getLanguage(lang)) {
            hljs.highlightElement(code);
          } else if (!lang) {
            const result = hljs.highlightAuto(code.textContent);
            code.innerHTML = result.value;
            code.classList.add("hljs");
          }
        } catch (err) {
          console.warn("highlight failed", err);
        }
      }
      const head = document.createElement("div");
      head.className = "code-head";
      const label = document.createElement("span");
      label.className = "code-lang";
      label.textContent = lang ? lang.toUpperCase() : "CODE";
      const copy = document.createElement("button");
      copy.type = "button";
      copy.className = "code-copy";
      copy.textContent = "复制";
      copy.addEventListener("click", async (event) => {
        event.stopPropagation();
        await copyText(code.textContent || "");
        copy.textContent = "已复制";
        setTimeout(() => { copy.textContent = "复制"; }, 1400);
      });
      head.append(label, copy);
      pre.prepend(head);
    });
  }

  function buildToc() {
    toc.replaceChildren();
    state.tocLinks = [];
    const headings = $$("h1, h2, h3, h4, h5, h6", article);
    $("#toc-count").textContent = headings.length ? String(headings.length) : "";
    if (!headings.length) {
      const empty = document.createElement("div");
      empty.className = "toc-empty";
      empty.textContent = "本文档没有标题";
      toc.append(empty);
      return;
    }
    for (const heading of headings) {
      const level = Number(heading.tagName.slice(1));
      if (level > 4) continue;
      const link = document.createElement("a");
      link.className = `toc-item level-${level}`;
      link.href = `#${encodeURIComponent(heading.id)}`;
      link.dataset.target = heading.id;
      link.textContent = heading.textContent.replace(/^#\s*/, "").trim() || heading.id;
      link.title = link.textContent;
      link.addEventListener("click", (event) => {
        event.preventDefault();
        heading.scrollIntoView({ behavior: "smooth", block: "start" });
        if (window.innerWidth <= 1080) toggleToc(false);
      });
      toc.append(link);
      state.tocLinks.push({ id: heading.id, level, link });
    }
    updateActiveHeading();
  }

  function updateActiveHeading() {
    if (!state.tocLinks.length) return;
    const offset = 220;
    let active = state.tocLinks[0].id;
    for (const item of state.tocLinks) {
      const heading = document.getElementById(item.id);
      if (!heading) continue;
      if (heading.getBoundingClientRect().top <= offset) active = item.id;
      else break;
    }
    if (state.activeId === active) return;
    state.activeId = active;
    for (const item of state.tocLinks) {
      item.link.classList.toggle("active", item.id === active);
    }
    const activeLink = state.tocLinks.find((item) => item.id === active)?.link;
    if (activeLink) {
      const panel = $("#toc");
      const top = activeLink.offsetTop;
      if (top < panel.scrollTop || top > panel.scrollTop + panel.clientHeight - 30) {
        panel.scrollTo({ top: Math.max(0, top - panel.clientHeight / 2), behavior: "smooth" });
      }
    }
  }

  function updateScrollUI() {
    const scrollTop = window.scrollY || document.documentElement.scrollTop;
    const max = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
    const percent = max > 0 ? Math.min(1, scrollTop / max) : 0;
    const progress = $("#progress");
    if (progress) progress.style.width = `${percent * 100}%`;
    const bar = $("#toc-progress-bar");
    if (bar) bar.style.width = `${percent * 100}%`;
    const percentText = $("#toc-percent");
    if (percentText) percentText.textContent = `${Math.round(percent * 100)}%`;
    $("#backtop")?.classList.toggle("show", scrollTop > 520);
    updateActiveHeading();
  }

  function renderMath(target = article) {
    if (!window.renderMathInElement) return;
    try {
      renderMathInElement(target, {
        delimiters: [
          { left: "$$", right: "$$", display: true },
          { left: "\\[", right: "\\]", display: true },
          { left: "$", right: "$", display: false },
          { left: "\\(", right: "\\)", display: false },
        ],
        throwOnError: false,
        strict: false,
        output: "mathml",
        ignoredTags: ["script", "noscript", "style", "textarea", "pre", "code", "option"],
      });
    } catch (err) {
      console.warn("math render failed", err);
    }
  }

  async function renderMermaid(target = article, tokenKey = "renderToken") {
    const token = ++state[tokenKey];
    const boxes = $$(".mermaid-box", target);
    if (!boxes.length) return;
    if (!window.mermaid) {
      boxes.forEach((box) => {
        box.textContent = box.dataset.code || "";
        box.classList.add("mermaid-error");
      });
      return;
    }
    try {
      mermaid.initialize({
        startOnLoad: false,
        securityLevel: "strict",
        theme: state.resolvedTheme === "dark" ? "dark" : "default",
        fontFamily: getComputedStyle(root).getPropertyValue("--font-ui").trim() || "sans-serif",
      });
    } catch (err) {
      console.warn("mermaid init failed", err);
    }
    for (const box of boxes) {
      if (token !== state[tokenKey]) return;
      const holder = box.querySelector(".mermaid-diagram") || box;
      const code = box.dataset.code || "";
      try {
        const id = `mdview-mermaid-${++state.mermaidSeq}`;
        const result = await mermaid.render(id, code);
        if (token !== state[tokenKey]) return;
        holder.innerHTML = result.svg;
        box.classList.remove("mermaid-error");
      } catch (err) {
        if (token !== state[tokenKey]) return;
        box.classList.add("mermaid-error");
        holder.textContent = `Mermaid 渲染失败：${err && err.message ? err.message : err}`;
      }
    }
  }

  function updateStatus() {
    if (!state.doc) return;
    const doc = state.doc;
    const headingCount = $$("h1, h2, h3, h4, h5, h6", article).length;
    $("#status-file").textContent = doc.file_name;
    $("#status-file").title = doc.path;
    const parts = [
      `${formatNumber(doc.word_count)} 字`,
      `${doc.read_minutes} 分钟`,
      doc.encoding,
      `${headingCount} 个标题`,
    ];
    if (doc.image_count) parts.push(`${doc.image_count} 张图片`);
    $("#status-meta").textContent = parts.join(" · ");
    const footer = $("#status-meta");
    if (footer) footer.title = `${doc.path}\n${formatNumber(doc.char_count)} 字符 · 更新于 ${formatDate(doc.modified_ms)}`;
  }

  function formatNumber(value) {
    return new Intl.NumberFormat("zh-CN").format(Number(value) || 0);
  }

  function formatDate(ms) {
    if (!ms) return "未知";
    try {
      return new Date(ms).toLocaleString("zh-CN", { hour12: false });
    } catch {
      return "未知";
    }
  }

  function renderDocument(doc, options = {}) {
    if (!doc || typeof doc !== "object") return;
    if (options.fromEdit) teardownEditUi();
    if (state.edit && !options.fromEdit) return;
    const previousMax = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
    const previousRatio = previousMax > 0 ? window.scrollY / previousMax : 0;
    state.doc = doc;
    state.activeId = "";
    state.renderToken += 1;
    const template = document.createElement("template");
    template.innerHTML = doc.html || "";
    sanitize(template.content);
    article.replaceChildren(template.content);
    if (!article.childNodes.length) {
      const empty = document.createElement("p");
      empty.className = "empty-doc";
      empty.textContent = "这个文档是空的。";
      article.append(empty);
    }
    decorateArticle();
    buildToc();
    updateStatus();
    renderMath();
    renderMermaid();
    body.classList.remove("empty");
    body.classList.add("reading");
    $("#doc-title").textContent = doc.title || doc.file_name || "Markdown";
    $("#doc-title").title = doc.title || "";
    $("#doc-path").textContent = doc.dir || doc.path || "";
    $("#doc-path").title = doc.path || "";
    document.title = `${doc.title || doc.file_name || "Markdown"} — mdview`;
    $("#welcome-error").hidden = true;
    recordRecent(doc.path);
    showChrome();
    const restored = options.preserveScroll ? previousRatio : 0;
    requestAnimationFrame(() => {
      const max = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
      window.scrollTo({ top: options.preserveScroll ? restored * max : 0, behavior: "auto" });
      updateScrollUI();
      if (!options.preserveScroll) {
        const resetTop = () => window.scrollTo({ top: 0, behavior: "auto" });
        setTimeout(resetTop, 180);
        setTimeout(resetTop, 700);
      }
    });
    if (options.preserveScroll && !options.silent) toast("文件已更新");
  }

  async function copyText(text) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      try {
        const area = document.createElement("textarea");
        area.value = text;
        area.style.position = "fixed";
        area.style.opacity = "0";
        document.body.append(area);
        area.focus();
        area.select();
        const ok = document.execCommand("copy");
        area.remove();
        return ok;
      } catch {
        toast("复制失败，请手动选择文本", "error");
        return false;
      }
    }
  }

  function showContextMenu(event) {
    if (state.edit) {
      event.preventDefault();
      return;
    }
    if (event.target.closest("input, textarea, select")) return;
    event.preventDefault();
    const menu = $("#context-menu");
    updateSettingsUI();
    menu.classList.add("open");
    const width = 260;
    const rect = menu.getBoundingClientRect();
    const height = Math.min(rect.height, window.innerHeight - 16);
    const x = Math.min(event.clientX, window.innerWidth - width - 10);
    const y = Math.min(event.clientY, window.innerHeight - height - 10);
    menu.style.left = `${Math.max(8, x)}px`;
    menu.style.top = `${Math.max(8, y)}px`;
  }

  function hideContextMenu() {
    $("#context-menu")?.classList.remove("open");
  }

  function openDrawer() {
    hideContextMenu();
    $("#drawer-mask").classList.add("open");
    $("#drawer").classList.add("open");
    body.classList.add("chrome-visible");
    showChrome(true);
  }

  function closeDrawer() {
    $("#drawer-mask").classList.remove("open");
    $("#drawer").classList.remove("open");
    scheduleChromeHide();
  }

  function showShortcuts() {
    $("#modal-mask").classList.add("open");
  }

  function closeModal() {
    $("#modal-mask").classList.remove("open");
  }

  function openLightbox(src, alt) {
    const lightbox = $("#lightbox");
    const image = $("#lightbox-img");
    if (!lightbox || !image) return;
    image.src = src;
    image.alt = alt || "";
    lightbox.classList.add("open");
  }

  function closeLightbox() {
    $("#lightbox")?.classList.remove("open");
    const image = $("#lightbox-img");
    if (image) image.src = "";
  }

  function handleAction(action) {
    switch (action) {
      case "new-window":
        ipc("newWindow");
        break;
      case "open-new":
        ipc("openNew");
        break;
      case "edit-wysiwyg":
        enterWysiwyg();
        break;
      case "edit-split":
        enterSplit();
        break;
      case "open":
        ipc("open");
        break;
      case "reload":
        ipc("reload");
        break;
      case "toc":
        toggleToc();
        break;
      case "export":
        ipc("exportHtml");
        break;
      case "export-pdf":
        ipc("exportPdf");
        break;
      case "print":
        ipc("printDialog");
        break;
      case "reveal":
        if (state.doc) ipc("reveal", { path: state.doc.path });
        break;
      case "copy-path":
        if (state.doc) copyText(state.doc.path).then((ok) => ok && toast("路径已复制"));
        break;
      case "shortcuts":
        showShortcuts();
        break;
      case "settings":
        openDrawer();
        break;
      case "close":
        ipc("close");
        break;
      case "register":
        ipc("register");
        toast("正在关联 .md 文件…");
        break;
      case "unregister":
        ipc("unregister");
        toast("正在解除 .md 文件关联…");
        break;
      case "open-config":
        ipc("openConfig");
        break;
      case "website":
        ipc("openExternal", { url: "https://www.mdview.cn/" });
        break;
      default:
        break;
    }
  }

  function bindEvents() {
    $("#btn-new").addEventListener("click", () => ipc("newWindow"));
    $("#btn-toc").addEventListener("click", () => toggleToc());
    $("#btn-open").addEventListener("click", () => ipc("open"));
    $("#btn-theme").addEventListener("click", toggleTheme);
    $("#btn-settings").addEventListener("click", openDrawer);
    $("#btn-edit").addEventListener("click", enterWysiwyg);
    $("#open-main").addEventListener("click", () => ipc("open"));
    $("#open-secondary").addEventListener("click", showShortcuts);
    $("#clear-recents").addEventListener("click", clearRecents);
    $("#toc-close").addEventListener("click", () => toggleToc(false));
    $("#backtop").addEventListener("click", () => window.scrollTo({ top: 0, behavior: "smooth" }));
    $("#wysiwyg-save").addEventListener("click", () => performEditSave(false));
    $("#wysiwyg-finish").addEventListener("click", finishEdit);
    $("#split-save").addEventListener("click", () => performEditSave(false));
    $("#split-finish").addEventListener("click", finishEdit);

    article.addEventListener("input", () => {
      if (state.edit?.mode === "wysiwyg") scheduleAutoSave();
    });
    article.addEventListener("paste", (event) => {
      if (state.edit?.mode !== "wysiwyg") return;
      event.preventDefault();
      const text = event.clipboardData?.getData("text/plain") || "";
      document.execCommand("insertText", false, text);
    });

    const sourceEditor = $("#source-editor");
    sourceEditor.addEventListener("input", schedulePreview);
    sourceEditor.addEventListener("scroll", () => {
      if (state.edit?.mode === "split") syncSplitScroll(sourceEditor, $("#preview-scroll"));
    });
    sourceEditor.addEventListener("keydown", (event) => {
      if (event.key === "Tab") {
        event.preventDefault();
        const start = sourceEditor.selectionStart;
        const end = sourceEditor.selectionEnd;
        sourceEditor.setRangeText("  ", start, end, "end");
        schedulePreview();
      }
    });
    $("#preview-scroll").addEventListener("scroll", () => {
      if (state.edit?.mode === "split") syncSplitScroll($("#preview-scroll"), sourceEditor);
    });
    $("#preview-content").addEventListener("click", (event) => {
      if (event.target.closest("a")) event.preventDefault();
    });

    const topbar = $(".topbar");
    topbar.addEventListener("mouseenter", () => showChrome(true));
    topbar.addEventListener("mouseleave", scheduleChromeHide);
    topbar.addEventListener("focusin", () => showChrome(true));

    document.addEventListener("mousemove", (event) => {
      if (!body.classList.contains("reading")) return;
      if (event.clientY <= 82) showChrome();
      else scheduleChromeHide();
    }, { passive: true });

    window.addEventListener("scroll", () => {
      updateScrollUI();
      if (body.classList.contains("reading")) scheduleChromeHide();
      hideContextMenu();
    }, { passive: true });
    window.addEventListener("resize", () => { hideContextMenu(); updateScrollUI(); });

    document.addEventListener("contextmenu", showContextMenu);
    document.addEventListener("click", (event) => {
      if (!event.target.closest("#context-menu")) hideContextMenu();
    });

    $("#context-menu").addEventListener("click", (event) => {
      const button = event.target.closest("[data-action]");
      if (!button) return;
      const action = button.dataset.action;
      if (action === "theme") {
        commitSettings({ theme: button.dataset.value });
        updateSettingsUI();
        return;
      }
      if (action === "accent") {
        commitSettings({ accent: button.dataset.value });
        updateSettingsUI();
        return;
      }
      if (action === "font-plus" || action === "font-minus" || action === "font-reset") {
        let next = state.settings.font_size;
        if (action === "font-plus") next += 1;
        if (action === "font-minus") next -= 1;
        if (action === "font-reset") next = 16;
        commitSettings({ font_size: Math.max(13, Math.min(24, next)) });
        updateSettingsUI();
        return;
      }
      hideContextMenu();
      handleAction(action);
    });

    $("#drawer-mask").addEventListener("click", closeDrawer);
    $("#drawer-close").addEventListener("click", closeDrawer);
    $$("[data-setting-theme]").forEach((button) => {
      button.addEventListener("click", () => commitSettings({ theme: button.dataset.settingTheme }));
    });
    $$("[data-setting-width]").forEach((button) => {
      button.addEventListener("click", () => commitSettings({ reading_width: Number(button.dataset.settingWidth) }));
    });
    $$("[data-setting-font]").forEach((button) => {
      button.addEventListener("click", () => commitSettings({ font: button.dataset.settingFont }));
    });
    $$(".swatch[data-accent]").forEach((button) => {
      button.addEventListener("click", () => commitSettings({ accent: button.dataset.accent }));
    });
    $("#font-range").addEventListener("input", (event) => {
      commitSettings({ font_size: Number(event.target.value) });
    });
    $("#toc-toggle").addEventListener("change", (event) => {
      toggleToc(event.target.checked);
    });
    $("#register-btn").addEventListener("click", () => handleAction("register"));
    $("#unregister-btn").addEventListener("click", () => handleAction("unregister"));
    $("#config-btn").addEventListener("click", () => handleAction("open-config"));
    $("#website-btn").addEventListener("click", () => handleAction("website"));
    $("#drawer-shortcuts").addEventListener("click", showShortcuts);

    $("#modal-mask").addEventListener("click", (event) => {
      if (event.target === $("#modal-mask")) closeModal();
    });
    $("#modal-close").addEventListener("click", closeModal);

    $("#lightbox").addEventListener("click", (event) => {
      if (event.target === $("#lightbox")) closeLightbox();
    });

    article.addEventListener("click", (event) => {
      if (state.edit) {
        if (event.target.closest("a")) event.preventDefault();
        return;
      }
      const image = event.target.closest("img");
      if (image) {
        event.preventDefault();
        openLightbox(image.currentSrc || image.src, image.alt);
        return;
      }
      const link = event.target.closest("a");
      if (!link) return;
      const href = link.getAttribute("href");
      if (!href) return;
      if (href.startsWith("#")) {
        event.preventDefault();
        const id = decodeURIComponent(href.slice(1));
        const target = document.getElementById(id);
        if (target) target.scrollIntoView({ behavior: "smooth", block: "start" });
        return;
      }
      if (href.startsWith("http://") || href.startsWith("https://") || href.startsWith("mailto:")) {
        event.preventDefault();
        ipc("openExternal", { url: href });
        return;
      }
      event.preventDefault();
      ipc("openLink", { href, base: state.doc?.dir || "" });
    });

    mqDark.addEventListener("change", () => {
      if (state.settings.theme !== "system") return;
      applySettings({ silentTheme: true });
      if (state.doc) renderMermaid();
    });

    document.addEventListener("keydown", (event) => {
      const target = event.target;
      const mod = event.ctrlKey || event.metaKey;
      const key = event.key.toLowerCase();
      const inField = target instanceof Element
        && (target.matches("input, textarea, select") || target.isContentEditable);

      if (state.edit) {
        if (event.key === "Escape") {
          event.preventDefault();
          finishEdit();
          return;
        }
        if (event.key === "F2" || event.key === "F3") {
          event.preventDefault();
          finishEdit();
          return;
        }
        if (mod && key === "s") {
          event.preventDefault();
          performEditSave(false);
          return;
        }
        return;
      }

      if (event.key === "Escape") {
        if ($("#lightbox").classList.contains("open")) { closeLightbox(); return; }
        if ($("#modal-mask").classList.contains("open")) { closeModal(); return; }
        if ($("#drawer").classList.contains("open")) { closeDrawer(); return; }
        if ($("#context-menu").classList.contains("open")) { hideContextMenu(); return; }
        if (!inField) ipc("close");
        return;
      }
      if (event.key === "F1" || (event.key === "?" && !mod)) {
        event.preventDefault();
        showShortcuts();
        return;
      }
      if (event.key === "F2") {
        event.preventDefault();
        enterWysiwyg();
        return;
      }
      if (event.key === "F3") {
        event.preventDefault();
        enterSplit();
        return;
      }
      if (inField) return;
      if (!mod) return;
      if (key === "n") { event.preventDefault(); ipc("newWindow"); }
      else if (key === "o") {
        event.preventDefault();
        ipc(event.shiftKey ? "openNew" : "open");
      }
      else if (key === "r") { event.preventDefault(); ipc("reload"); }
      else if (key === "p") {
        event.preventDefault();
        ipc(event.shiftKey ? "printDialog" : "exportPdf");
      }
      else if (key === "e") { event.preventDefault(); ipc("exportHtml"); }
      else if (key === "b") { event.preventDefault(); toggleToc(); }
      else if (key === "d") { event.preventDefault(); toggleTheme(); }
      else if (key === ",") { event.preventDefault(); openDrawer(); }
      else if (key === "w") { event.preventDefault(); ipc("close"); }
      else if (key === "=" || key === "+") {
        event.preventDefault();
        commitSettings({ font_size: Math.min(24, state.settings.font_size + 1) });
      } else if (key === "-") {
        event.preventDefault();
        commitSettings({ font_size: Math.max(13, state.settings.font_size - 1) });
      } else if (key === "0") {
        event.preventDefault();
        commitSettings({ font_size: 16 });
      }
    });

    window.addEventListener("dragover", (event) => {
      event.preventDefault();
      $(".drop-zone")?.classList.add("dragging");
    });
    window.addEventListener("dragleave", () => $(".drop-zone")?.classList.remove("dragging"));
    window.addEventListener("drop", (event) => {
      event.preventDefault();
      $(".drop-zone")?.classList.remove("dragging");
      const file = event.dataTransfer?.files?.[0];
      if (file && file.path) ipc("openPath", { path: file.path });
    });
  }

  function init() {
    if ("scrollRestoration" in history) history.scrollRestoration = "manual";
    window.addEventListener("error", (event) => {      ipc("jsError", {
        message: String(event.message || "unknown error"),
        source: String(event.filename || ""),
        line: Number(event.lineno || 0),
      });
    });
    window.addEventListener("unhandledrejection", (event) => {
      ipc("jsError", { message: `Promise rejection: ${String(event.reason)}` });
    });
    applySettings({ silentTheme: true });
    renderRecent();
    bindEvents();
    updateScrollUI();
    window.mdview = {
      loadDoc(doc, options) {
        renderDocument(doc, options || {});
      },
      applySettings(nextSettings) {
        if (!nextSettings || typeof nextSettings !== "object") return;
        state.settings = Object.assign({}, defaults, nextSettings);
        if (!Array.isArray(state.settings.recent)) state.settings.recent = [];
        applySettings();
        renderRecent();
      },
      editSaved: onEditSaved,
      editError: onEditError,
      preview: onPreview,
      showError,
      toast,
      settings() {
        return state.settings;
      },
    };
    ipc("ready");
    setTimeout(() => {
      if (!state.doc) showChrome(true);
    }, 100);
  }

  ready(init);
})();
