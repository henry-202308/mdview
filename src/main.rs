#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod assoc;
mod document;
mod settings;

use document::LoadedDoc;
use percent_encoding::percent_decode_str;
use serde_json::Value;
use settings::Settings;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tao::dpi::{LogicalSize, PhysicalSize};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Icon, Window, WindowBuilder, WindowId};
use wry::http::{header::CONTENT_TYPE, Request, Response};
use wry::{DragDropEvent, WebContext, WebView, WebViewBuilder};
#[cfg(windows)]
use wry::{WebViewBuilderExtWindows, WebViewExtWindows};

const APP_TITLE: &str = "mdview — Markdown 阅读器";
const WATCH_INTERVAL: Duration = Duration::from_millis(700);
const APP_ICON_SIZE: i32 = 64;

enum UserEvent {
    Ipc(WindowId, String),
    Drop(WindowId, Vec<PathBuf>),
    PdfDone(WindowId, Result<PathBuf, String>),
}

enum Cli {
    Run(Option<PathBuf>),
    Register,
    Unregister,
    Help,
    Version,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_cli(&args) {
        Cli::Run(path) => {
            if let Err(err) = run_app(path) {
                show_dialog("mdview 启动失败", &err, true);
            }
        }
        Cli::Register => {
            let (title, message, error): (&str, String, bool) = match assoc::register() {
                Ok(()) => (
                    "mdview",
                    "已关联 .md 文件（当前用户）。现在双击 Markdown 文件即可用 mdview 打开。"
                        .to_string(),
                    false,
                ),
                Err(err) => ("mdview", format!("关联失败：{err}"), true),
            };
            report_cli(title, &message, error);
        }
        Cli::Unregister => {
            let (title, message, error): (&str, String, bool) = match assoc::unregister() {
                Ok(()) => (
                    "mdview",
                    "已解除本程序创建的 .md 文件关联。".to_string(),
                    false,
                ),
                Err(err) => ("mdview", format!("解除关联失败：{err}"), true),
            };
            report_cli(title, &message, error);
        }
        Cli::Help => report_cli("mdview", &help_text(), false),
        Cli::Version => report_cli(
            "mdview",
            &format!(
                "mdview {} · Rust + WebView2\n{}",
                env!("CARGO_PKG_VERSION"),
                "本地 Markdown 阅读器"
            ),
            false,
        ),
    }
}

fn parse_cli(args: &[String]) -> Cli {
    let mut path = None;
    let mut positional_only = false;
    for arg in args {
        if positional_only {
            if path.is_none() {
                path = Some(PathBuf::from(arg));
            }
            continue;
        }
        match arg.as_str() {
            "--" => positional_only = true,
            "--register" | "--associate" => return Cli::Register,
            "--unregister" | "--unassociate" => return Cli::Unregister,
            "-h" | "--help" => return Cli::Help,
            "-V" | "--version" => return Cli::Version,
            value if value.starts_with('-') && !Path::new(value).exists() => {
                return Cli::Help;
            }
            value => {
                if path.is_none() {
                    path = Some(PathBuf::from(value));
                }
            }
        }
    }
    Cli::Run(path)
}

fn help_text() -> String {
    format!(
        "mdview {} — 使用 Rust + Windows WebView2 的 Markdown 阅读器\n\n\
         用法：\n  mdview [文件.md]\n\n\
         选项：\n  --register, --associate      关联 .md 文件（当前用户）\n  \
         --unregister                解除本程序创建的关联\n  \
         -h, --help                  显示帮助\n  \
         -V, --version               显示版本\n\n\
         快捷键：Ctrl+N 新建窗口 · Ctrl+O 打开 · Ctrl+P 导出 PDF\n\
         F2 就地编辑 · F3 分屏编辑 · Ctrl+S 保存 · Esc 退出编辑/关闭",
        env!("CARGO_PKG_VERSION")
    )
}

fn report_cli(title: &str, message: &str, error: bool) {
    println!("{message}");
    #[cfg(not(debug_assertions))]
    show_dialog(title, message, error);
    #[cfg(debug_assertions)]
    let _ = (title, error);
}

fn show_dialog(title: &str, message: &str, error: bool) {
    let level = if error {
        rfd::MessageLevel::Error
    } else {
        rfd::MessageLevel::Info
    };
    let _ = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(level)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

fn run_app(initial: Option<PathBuf>) -> Result<(), String> {
    let mut event_loop_builder = EventLoopBuilder::<UserEvent>::with_user_event();
    let event_loop = event_loop_builder.build();
    let proxy = event_loop.create_proxy();
    let settings = Settings::load();
    let webview_version = wry::webview_version().unwrap_or_else(|_| "未知".to_string());
    let mut app = App::new(proxy, settings, webview_version);
    app.create_window(&event_loop, initial)?;

    let mut next_check = Instant::now() + WATCH_INTERVAL;
    event_loop.run(move |event, target, control_flow| {
        *control_flow = ControlFlow::WaitUntil(next_check);
        match event {
            Event::NewEvents(StartCause::Init) => {
                if let Some(window) = app.windows.values().next() {
                    window.window.set_focus();
                    let _ = window.webview.focus();
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } => {
                app.close_window(window_id, control_flow);
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Destroyed,
                ..
            } => {
                app.windows.remove(&window_id);
                if app.windows.is_empty() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(UserEvent::Ipc(window_id, body)) => {
                app.handle_ipc(window_id, &body, target, control_flow);
            }
            Event::UserEvent(UserEvent::Drop(window_id, paths)) => {
                if let Some(window) = app.windows.get_mut(&window_id) {
                    if window.editing {
                        window.toast("请先完成编辑（Esc），再拖入其他文件", true);
                    } else if let Some(path) = pick_dropped_path(&paths) {
                        window.open_path(&path, false, false, &mut app.settings);
                    } else if !paths.is_empty() {
                        window.toast("请拖入 Markdown 文件（.md / .markdown / .txt）", true);
                    }
                }
            }
            Event::UserEvent(UserEvent::PdfDone(window_id, outcome)) => {
                if let Some(window) = app.windows.get(&window_id) {
                    match outcome {
                        Ok(path) => window.toast(&format!("PDF 已导出：{}", path.display()), false),
                        Err(err) => window.toast(&err, true),
                    }
                }
            }
            Event::MainEventsCleared => {
                let now = Instant::now();
                if now >= next_check {
                    app.tick();
                    next_check = now + WATCH_INTERVAL;
                    *control_flow = ControlFlow::WaitUntil(next_check);
                }
            }
            Event::LoopDestroyed => {
                let _ = app.settings.save();
            }
            _ => {}
        }
    });
}

struct App {
    settings: Settings,
    windows: HashMap<WindowId, WindowState>,
    proxy: EventLoopProxy<UserEvent>,
    webview_version: String,
    web_context: Option<WebContext>,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>, settings: Settings, webview_version: String) -> Self {
        let data_dir = webview_data_dir();
        Self {
            settings,
            windows: HashMap::new(),
            proxy,
            webview_version,
            web_context: data_dir.map(|path| WebContext::new(Some(path))),
        }
    }

    fn create_window(
        &mut self,
        target: &EventLoopWindowTarget<UserEvent>,
        initial: Option<PathBuf>,
    ) -> Result<(), String> {
        let settings = self.settings.clone();
        let proxy = self.proxy.clone();
        let version = self.webview_version.clone();
        let window = WindowState::new(
            target,
            &proxy,
            &settings,
            &version,
            initial,
            self.web_context.as_mut(),
        )?;
        let id = window.window.id();
        window.window.set_focus();
        let _ = window.webview.focus();
        self.windows.insert(id, window);
        Ok(())
    }

    fn close_window(&mut self, id: WindowId, control_flow: &mut ControlFlow) {
        self.windows.remove(&id);
        if self.windows.is_empty() {
            let _ = self.settings.save();
            *control_flow = ControlFlow::Exit;
        }
    }

    fn handle_ipc(
        &mut self,
        window_id: WindowId,
        body: &str,
        target: &EventLoopWindowTarget<UserEvent>,
        control_flow: &mut ControlFlow,
    ) {
        let Ok(value) = serde_json::from_str::<Value>(body) else {
            return;
        };
        let Some(cmd) = value.get("cmd").and_then(Value::as_str) else {
            return;
        };
        match cmd {
            "newWindow" => {
                if let Err(err) = self.create_window(target, None) {
                    self.toast(window_id, &format!("无法新建窗口：{err}"), true);
                }
            }
            "openNew" => {
                let dir = self
                    .windows
                    .get(&window_id)
                    .and_then(WindowState::default_dir);
                if let Some(path) = pick_markdown(dir.as_deref()) {
                    if let Err(err) = self.create_window(target, Some(path)) {
                        self.toast(window_id, &format!("无法新建窗口：{err}"), true);
                    }
                }
            }
            "openPathNew" => {
                if let Some(path) = value.get("path").and_then(Value::as_str) {
                    if let Err(err) = self.create_window(target, Some(PathBuf::from(path))) {
                        self.toast(window_id, &format!("无法新建窗口：{err}"), true);
                    }
                }
            }
            "saveSettings" => {
                if let Some(settings_value) = value.get("settings") {
                    if let Ok(mut next) = serde_json::from_value::<Settings>(settings_value.clone())
                    {
                        next.sanitize();
                        self.settings = next;
                        if let Err(err) = self.settings.save() {
                            eprintln!("save settings: {err}");
                        }
                        self.broadcast_settings();
                    }
                }
            }
            _ => {
                let proxy = self.proxy.clone();
                let close = {
                    let settings = &mut self.settings;
                    if let Some(window) = self.windows.get_mut(&window_id) {
                        window.handle_ipc(cmd, &value, settings, &proxy)
                    } else {
                        false
                    }
                };
                if close {
                    self.close_window(window_id, control_flow);
                }
            }
        }
    }

    fn broadcast_settings(&self) {
        let Ok(json) = serde_json::to_string(&self.settings) else {
            return;
        };
        let script = format!("window.mdview && window.mdview.applySettings({json});");
        for window in self.windows.values() {
            let _ = window.webview.evaluate_script(&script);
        }
    }

    fn toast(&self, id: WindowId, message: &str, error: bool) {
        if let Some(window) = self.windows.get(&id) {
            window.toast(message, error);
        }
    }

    fn tick(&mut self) {
        for window in self.windows.values_mut() {
            window.tick();
        }
    }
}

struct WindowState {
    window: Window,
    webview: WebView,
    current: Option<LoadedDoc>,
    pending: Option<PathBuf>,
    ready: bool,
    editing: bool,
}

impl WindowState {
    fn new(
        target: &EventLoopWindowTarget<UserEvent>,
        proxy: &EventLoopProxy<UserEvent>,
        settings: &Settings,
        webview_version: &str,
        initial: Option<PathBuf>,
        context: Option<&mut WebContext>,
    ) -> Result<Self, String> {
        let window = WindowBuilder::new()
            .with_title(APP_TITLE)
            .with_inner_size(LogicalSize::new(640.0, 400.0))
            .with_min_inner_size(LogicalSize::new(420.0, 320.0))
            .with_window_icon(make_icon())
            .with_visible(false)
            .build(target)
            .map_err(|err| format!("无法创建窗口：{err}"))?;
        apply_window_size(&window);

        let window_id = window.id();
        let shell = Arc::new(assets::shell_html(
            settings,
            &settings.recent,
            env!("CARGO_PKG_VERSION"),
            webview_version,
        ));
        let ipc_proxy = proxy.clone();
        let drop_proxy = proxy.clone();
        let protocol_shell = Arc::clone(&shell);
        let mut builder = if let Some(context) = context {
            WebViewBuilder::with_web_context(context)
        } else {
            WebViewBuilder::new()
        }
        .with_custom_protocol("mdview".to_string(), move |_id, request| {
            protocol_response(request, protocol_shell.as_str())
        })
        .with_url("mdview://localhost/index.html")
        .with_background_color((245, 246, 247, 255))
        .with_ipc_handler(move |request: wry::http::Request<String>| {
            let _ = ipc_proxy.send_event(UserEvent::Ipc(window_id, request.body().to_string()));
        })
        .with_drag_drop_handler(move |event| {
            if let DragDropEvent::Drop { paths, .. } = event {
                let _ = drop_proxy.send_event(UserEvent::Drop(window_id, paths));
            }
            true
        });
        #[cfg(windows)]
        {
            builder = builder.with_browser_accelerator_keys(false);
            #[cfg(debug_assertions)]
            {
                builder = builder.with_additional_browser_args(
                    "--remote-debugging-port=9222 --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
                );
            }
        }
        let webview = builder.build(&window).map_err(|err| {
            format!(
                "无法创建 WebView2 控件。\n\n请确认系统已安装 Microsoft Edge WebView2 Runtime。\n\n{err}"
            )
        })?;
        window.set_visible(true);
        let _ = webview.focus();

        Ok(Self {
            window,
            webview,
            current: None,
            pending: initial,
            ready: false,
            editing: false,
        })
    }

    fn handle_ipc(
        &mut self,
        cmd: &str,
        value: &Value,
        settings: &mut Settings,
        proxy: &EventLoopProxy<UserEvent>,
    ) -> bool {
        match cmd {
            "ready" => {
                if self.ready {
                    return false;
                }
                self.ready = true;
                if let Some(path) = self.pending.take() {
                    self.open_path(&path, false, false, settings);
                }
            }
            "open" => {
                if let Some(path) = pick_markdown(self.default_dir().as_deref()) {
                    self.open_path(&path, false, false, settings);
                }
            }
            "openPath" => {
                if let Some(path) = value.get("path").and_then(Value::as_str) {
                    self.open_path(Path::new(path), false, false, settings);
                }
            }
            "openLink" => {
                let href = value.get("href").and_then(Value::as_str).unwrap_or("");
                let base = value.get("base").and_then(Value::as_str).unwrap_or("");
                self.open_link(href, base, settings);
            }
            "openExternal" => {
                let url = value.get("url").and_then(Value::as_str).unwrap_or("");
                self.open_external(url);
            }
            "reveal" => {
                if let Some(path) = value.get("path").and_then(Value::as_str) {
                    reveal_in_explorer(Path::new(path));
                }
            }
            "printDialog" => {
                if let Err(err) = self.webview.print() {
                    self.toast(&format!("无法打开打印对话框：{err}"), true);
                }
            }
            "exportHtml" => {
                let theme = value
                    .get("theme")
                    .and_then(Value::as_str)
                    .unwrap_or("light");
                self.export_html(theme);
            }
            "exportPdf" => {
                self.export_pdf(proxy);
            }
            "reload" => {
                if let Some(path) = self.current.as_ref().map(|doc| doc.path.clone()) {
                    self.open_path(&path, true, false, settings);
                }
            }
            "setEditing" => {
                self.editing = value
                    .get("editing")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
            }
            "saveEdit" => {
                let markdown = value.get("markdown").and_then(Value::as_str).unwrap_or("");
                let body_only = value.get("mode").and_then(Value::as_str) == Some("body");
                let finish = value
                    .get("finish")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                self.save_edit(markdown, body_only, finish);
            }
            "finishEdit" => {
                self.editing = false;
                if let Some(doc) = &self.current {
                    self.send_document(doc, true, true, true);
                }
            }
            "renderPreview" => {
                let source = value.get("source").and_then(Value::as_str).unwrap_or("");
                self.render_preview(source);
            }
            "register" => match assoc::register() {
                Ok(()) => self.toast("已关联 .md 文件，双击即可打开", false),
                Err(err) => self.toast(&format!("关联失败：{err}"), true),
            },
            "unregister" => match assoc::unregister() {
                Ok(()) => self.toast("已解除 .md 文件关联", false),
                Err(err) => self.toast(&format!("解除关联失败：{err}"), true),
            },
            "openConfig" => {
                let dir = settings::config_dir();
                let _ = fs::create_dir_all(&dir);
                if let Err(err) = open::that(&dir) {
                    self.toast(&format!("无法打开设置目录：{err}"), true);
                }
            }
            "close" => return true,
            "jsError" => {
                let message = value
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown JS error");
                eprintln!("js error: {message}");
            }
            _ => {}
        }
        false
    }

    fn default_dir(&self) -> Option<PathBuf> {
        self.current
            .as_ref()
            .and_then(|doc| doc.path.parent().map(Path::to_path_buf))
            .or_else(|| std::env::current_dir().ok())
    }

    fn open_path(&mut self, path: &Path, preserve: bool, quiet: bool, settings: &mut Settings) {
        if !self.ready {
            self.pending = Some(path.to_path_buf());
            return;
        }
        match LoadedDoc::load(path) {
            Ok(doc) => {
                settings.record_recent(&doc.path);
                if let Err(err) = settings.save() {
                    eprintln!("save settings: {err}");
                }
                self.editing = false;
                self.window.set_title(&window_title(&doc.title));
                self.send_document(&doc, preserve, false, false);
                self.current = Some(doc);
                let _ = self.webview.focus();
            }
            Err(err) => {
                if quiet {
                    eprintln!("watch reload failed: {err}");
                    return;
                }
                self.toast(&format!("打开失败：{err}"), true);
                self.show_error(&format!("无法打开《{}》\n{}", path.display(), err));
            }
        }
    }

    fn send_document(&self, doc: &LoadedDoc, preserve: bool, silent: bool, from_edit: bool) {
        let Ok(payload) = serde_json::to_string(&doc.payload) else {
            return;
        };
        let script = format!(
            "window.mdview && window.mdview.loadDoc({payload}, {{preserveScroll:{},silent:{},fromEdit:{}}});",
            if preserve { "true" } else { "false" },
            if silent { "true" } else { "false" },
            if from_edit { "true" } else { "false" }
        );
        if let Err(err) = self.webview.evaluate_script(&script) {
            eprintln!("send document: {err}");
        }
    }

    fn save_edit(&mut self, markdown: &str, body_only: bool, finish: bool) {
        let Some(doc) = self.current.as_ref() else {
            self.toast("没有正在编辑的文档", true);
            return;
        };
        match document::save_markdown(doc, markdown, body_only) {
            Ok(next) => {
                let title = next.title.clone();
                let payload = serde_json::json!({
                    "wordCount": next.payload.word_count,
                    "charCount": next.payload.char_count,
                    "readMinutes": next.payload.read_minutes,
                    "modifiedMs": next.payload.modified_ms,
                    "raw": next.raw_source.clone(),
                    "title": title,
                });
                self.window.set_title(&window_title(&title));
                self.current = Some(next);
                if finish {
                    self.editing = false;
                    if let Some(doc) = &self.current {
                        self.send_document(doc, true, true, true);
                    }
                } else {
                    let script = format!("window.mdview && window.mdview.editSaved({payload});");
                    let _ = self.webview.evaluate_script(&script);
                }
            }
            Err(err) => {
                let message = serde_json::to_string(&format!("保存失败：{err}"))
                    .unwrap_or_else(|_| "\"保存失败\"".to_string());
                let script = format!("window.mdview && window.mdview.editError({message});");
                let _ = self.webview.evaluate_script(&script);
            }
        }
    }

    fn render_preview(&self, source: &str) {
        let Some(doc) = self.current.as_ref() else {
            return;
        };
        let base = doc.path.parent().unwrap_or_else(|| Path::new("."));
        let (html, word_count, char_count) = document::preview_html(source, base);
        let payload = serde_json::json!({
            "html": html,
            "wordCount": word_count,
            "charCount": char_count,
        });
        let script = format!("window.mdview && window.mdview.preview({payload});");
        let _ = self.webview.evaluate_script(&script);
    }

    fn tick(&mut self) {
        if self.editing {
            return;
        }
        let Some(doc) = self.current.as_ref() else {
            return;
        };
        if !doc.changed_on_disk() {
            return;
        }
        let path = doc.path.clone();
        let previous_html = doc.payload.html.clone();
        match LoadedDoc::load(&path) {
            Ok(next) => {
                if next.payload.html == previous_html {
                    self.current = Some(next);
                    return;
                }
                self.window.set_title(&window_title(&next.title));
                self.send_document(&next, true, false, false);
                self.current = Some(next);
            }
            Err(err) => {
                eprintln!("watch reload failed: {err}");
            }
        }
    }

    fn open_link(&mut self, href: &str, base: &str, settings: &mut Settings) {
        let href = href.trim();
        if href.is_empty() {
            return;
        }
        if href.starts_with("http://")
            || href.starts_with("https://")
            || href.starts_with("mailto:")
        {
            self.open_external(href);
            return;
        }
        let Some(path) = resolve_file_link(href, base) else {
            self.toast("无法解析这个链接", true);
            return;
        };
        if !path.exists() {
            self.toast(&format!("链接目标不存在：{}", path.display()), true);
            return;
        }
        if is_markdown_path(&path) {
            self.open_path(&path, false, false, settings);
        } else if let Err(err) = open::that(&path) {
            self.toast(&format!("无法打开链接：{err}"), true);
        }
    }

    fn open_external(&mut self, url: &str) {
        let trimmed = url.trim();
        if !(trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("mailto:"))
        {
            self.toast("仅支持打开 http / https / mailto 链接", true);
            return;
        }
        if let Err(err) = open::that(trimmed) {
            self.toast(&format!("无法打开链接：{err}"), true);
        }
    }

    fn export_html(&mut self, theme: &str) {
        let Some(doc) = self.current.as_ref() else {
            self.toast("请先打开一个文档", true);
            return;
        };
        let mut dialog = rfd::FileDialog::new()
            .add_filter("HTML 文件", &["html", "htm"])
            .set_title("导出单文件 HTML");
        if let Some(parent) = doc.path.parent() {
            dialog = dialog.set_directory(parent);
        }
        let stem = doc
            .path
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "markdown".to_string());
        dialog = dialog.set_file_name(format!("{stem}.html"));
        let Some(mut target) = dialog.save_file() else {
            return;
        };
        if target.extension().is_none() {
            target.set_extension("html");
        }
        let html = document::build_standalone_html(
            doc,
            assets::APP_CSS,
            assets::HIGHLIGHT_JS,
            assets::MERMAID_JS,
            assets::KATEX_JS,
            assets::AUTO_RENDER_JS,
            theme,
        );
        match fs::write(&target, html) {
            Ok(()) => {
                let name = target
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| target.to_string_lossy().to_string());
                self.toast(&format!("已导出 {name}"), false);
            }
            Err(err) => self.toast(&format!("导出失败：{err}"), true),
        }
    }

    fn export_pdf(&mut self, proxy: &EventLoopProxy<UserEvent>) {
        let Some(doc) = self.current.as_ref() else {
            self.toast("请先打开一个文档", true);
            return;
        };
        let target = unique_pdf_path(&doc.path);
        self.toast("正在生成 PDF…", false);
        #[cfg(windows)]
        {
            if let Err(err) = self.start_pdf(&target, proxy) {
                self.toast(&format!("PDF 导出失败：{err}"), true);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (target, proxy);
            self.toast("当前平台暂不支持无对话框 PDF 导出", true);
        }
    }

    #[cfg(windows)]
    fn start_pdf(&self, target: &Path, proxy: &EventLoopProxy<UserEvent>) -> Result<(), String> {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2PrintSettings, ICoreWebView2_7,
        };
        use webview2_com::PrintToPdfCompletedHandler;
        use windows_core::{Interface, HSTRING};

        let controller = self.webview.controller();
        let core = unsafe { controller.CoreWebView2() }
            .map_err(|err| format!("无法获取 WebView2 核心对象：{err}"))?;
        let core7: ICoreWebView2_7 = core
            .cast()
            .map_err(|err| format!("当前 WebView2 Runtime 不支持 PDF 导出：{err}"))?;
        let path = target.to_path_buf();
        let callback_path = path.clone();
        let window_id = self.window.id();
        let proxy = proxy.clone();
        let handler = PrintToPdfCompletedHandler::create(Box::new(move |result, success| {
            let outcome = if success && result.is_ok() {
                Ok(callback_path.clone())
            } else {
                Err(format!(
                    "WebView2 生成 PDF 失败{}",
                    result
                        .err()
                        .map(|err| format!("：{err}"))
                        .unwrap_or_default()
                ))
            };
            let _ = proxy.send_event(UserEvent::PdfDone(window_id, outcome));
            Ok(())
        }));
        let settings: Option<&ICoreWebView2PrintSettings> = None;
        let path_hstring = HSTRING::from(path.to_string_lossy().to_string());
        unsafe { core7.PrintToPdf(&path_hstring, settings, &handler) }
            .map_err(|err| format!("无法启动 PDF 打印：{err}"))
    }

    fn toast(&self, message: &str, error: bool) {
        let Ok(message) = serde_json::to_string(message) else {
            return;
        };
        let kind = if error { "'error'" } else { "''" };
        let script = format!("window.mdview && window.mdview.toast({message}, {kind});");
        let _ = self.webview.evaluate_script(&script);
    }

    fn show_error(&self, message: &str) {
        let Ok(message) = serde_json::to_string(message) else {
            return;
        };
        let script = format!("window.mdview && window.mdview.showError({message});");
        let _ = self.webview.evaluate_script(&script);
    }
}

fn apply_window_size(window: &Window) {
    if let Some(monitor) = window.current_monitor() {
        let size = monitor.size();
        let width = ((size.width as f64 * 0.88) as u32).clamp(900, 2400);
        let height = ((size.height as f64 * 0.86) as u32).clamp(620, 1400);
        window.set_inner_size(PhysicalSize::new(width, height));
    }
}

fn unique_pdf_path(markdown: &Path) -> PathBuf {
    let directory = markdown.parent().unwrap_or_else(|| Path::new("."));
    let stem = markdown
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "markdown".to_string());
    let first = markdown.with_extension("pdf");
    if !first.exists() {
        return first;
    }
    for index in 1..1000 {
        let candidate = directory.join(format!("{stem} ({index}).pdf"));
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!("{stem}.pdf"))
}

fn protocol_response(request: Request<Vec<u8>>, shell: &str) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path();
    match path {
        "/" | "/index.html" => Response::builder()
            .status(200)
            .header(CONTENT_TYPE, "text/html; charset=utf-8")
            .header("Cache-Control", "no-store")
            .header("X-Content-Type-Options", "nosniff")
            .body(Cow::Owned(shell.as_bytes().to_vec()))
            .unwrap_or_else(|_| plain_response(500, "Internal error")),
        "/local" => {
            let query = request.uri().query().unwrap_or("");
            let encoded = query.strip_prefix("path=").unwrap_or("");
            let decoded = percent_decode_str(encoded).decode_utf8_lossy();
            let image = PathBuf::from(decoded.as_ref());
            if !is_image_path(&image) {
                return plain_response(404, "Not found");
            }
            match fs::read(&image) {
                Ok(bytes) => Response::builder()
                    .status(200)
                    .header(CONTENT_TYPE, document::mime_for(&image))
                    .header("Cache-Control", "no-store")
                    .header("X-Content-Type-Options", "nosniff")
                    .body(Cow::Owned(bytes))
                    .unwrap_or_else(|_| plain_response(500, "Internal error")),
                Err(_) => plain_response(404, "Image not found"),
            }
        }
        _ => plain_response(404, "Not found"),
    }
}

fn plain_response(status: u16, message: &'static str) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("X-Content-Type-Options", "nosniff")
        .body(Cow::Borrowed(message.as_bytes()))
        .expect("valid static response")
}

fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "avif"
    )
}

fn window_title(title: &str) -> String {
    let first_line = title.lines().next().unwrap_or(title).trim();
    let mut shortened: String = first_line.chars().take(90).collect();
    if shortened.is_empty() {
        shortened = "Markdown".to_string();
    }
    format!("{shortened} — mdview")
}

fn pick_markdown(directory: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new()
        .add_filter(
            "Markdown",
            &["md", "markdown", "mdown", "mkd", "mdx", "txt"],
        )
        .add_filter("所有文件", &["*"])
        .set_title("打开 Markdown 文件");
    if let Some(directory) = directory {
        dialog = dialog.set_directory(directory);
    }
    dialog.pick_file()
}

fn pick_dropped_path(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|path| is_markdown_path(path)).cloned()
}

fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "md" | "markdown" | "mdown" | "mkd" | "mdx" | "txt" | "text"
    )
}

fn resolve_file_link(href: &str, base: &str) -> Option<PathBuf> {
    let href = href.split(['?', '#']).next().unwrap_or(href).trim();
    if href.is_empty() {
        return None;
    }
    let decoded = percent_decode_str(href).decode_utf8_lossy().to_string();
    let path = if let Some(rest) = decoded.strip_prefix("file://") {
        let trimmed = rest.trim_start_matches('/');
        if trimmed.len() >= 2 && trimmed.as_bytes()[1] == b':' {
            PathBuf::from(trimmed.replace('/', "\\"))
        } else {
            PathBuf::from(decoded.replace('/', "\\").trim_start_matches('\\'))
        }
    } else {
        PathBuf::from(decoded.replace('/', "\\"))
    };
    let resolved = if path.is_absolute() {
        path
    } else {
        Path::new(base).join(path)
    };
    Some(document::normalize_path(&resolved))
}

fn reveal_in_explorer(path: &Path) {
    let target = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut command = Command::new("explorer.exe");
        command.arg(format!("/select,{}", target.display()));
        command.creation_flags(CREATE_NO_WINDOW);
        let _ = command.spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = open::that(target);
    }
}

fn webview_data_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let probe = exe_dir.join(".mdview-write-probe");
    let writable = fs::write(&probe, b"probe").is_ok();
    let _ = fs::remove_file(&probe);
    if writable {
        return None;
    }

    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))?;
    let dir = base.join("mdview").join("WebView2");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn make_icon() -> Option<Icon> {
    let size = APP_ICON_SIZE;
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let qx = (px - 32.0).abs() - 18.0;
            let qy = (py - 32.0).abs() - 18.0;
            let distance =
                (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - 14.0;
            let alpha = (0.5 - distance).clamp(0.0, 1.0);
            if alpha <= 0.0 {
                continue;
            }
            let t = ((px + py) / 128.0).clamp(0.0, 1.0);
            let mut r = 9.0 + (5.0 - 9.0) * t;
            let mut g = 105.0 + (80.0 - 105.0) * t;
            let mut b = 218.0 + (174.0 - 218.0) * t;

            let mx = [18.0, 18.0, 32.0, 46.0];
            let my = [45.0, 19.0, 36.0, 45.0];
            let mut mark = f32::MAX;
            for index in 0..3 {
                mark = mark.min(segment_distance(
                    px,
                    py,
                    mx[index],
                    my[index],
                    mx[index + 1],
                    my[index + 1],
                ));
            }
            if mark < 3.2 {
                let coverage = (3.2 - mark).clamp(0.0, 1.0);
                r = r + (255.0 - r) * coverage;
                g = g + (255.0 - g) * coverage;
                b = b + (255.0 - b) * coverage;
            }
            let index = ((y * size + x) * 4) as usize;
            pixels[index] = r as u8;
            pixels[index + 1] = g as u8;
            pixels[index + 2] = b as u8;
            pixels[index + 3] = (alpha * 255.0) as u8;
        }
    }
    Icon::from_rgba(pixels, size as u32, size as u32).ok()
}

fn segment_distance(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((px - ax) * dx + (py - ay) * dy) / length).clamp(0.0, 1.0)
    };
    let nx = ax + t * dx;
    let ny = ay + t * dy;
    ((px - nx).powi(2) + (py - ny).powi(2)).sqrt()
}
