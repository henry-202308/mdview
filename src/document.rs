use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImageMode {
    /// Served through the `mdview://localhost/local` protocol handler.
    Protocol,
    /// Embedded in a single-file HTML export.
    DataUri,
}

const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'?')
    .add(b'+')
    .add(b'=');

#[derive(Clone, Debug, Default, Serialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentPayload {
    pub path: String,
    pub dir: String,
    pub file_name: String,
    pub title: String,
    pub html: String,
    pub raw: String,
    pub word_count: usize,
    pub char_count: usize,
    pub read_minutes: usize,
    pub encoding: String,
    pub modified_ms: u64,
    pub size: u64,
    pub image_count: usize,
    pub link_count: usize,
    pub front: FrontMatter,
}

pub struct LoadedDoc {
    pub path: PathBuf,
    pub title: String,
    pub payload: DocumentPayload,
    pub modified: SystemTime,
    pub source: String,
    pub raw_source: String,
    pub frontmatter_prefix: String,
}

impl LoadedDoc {
    pub fn load(path: &Path) -> Result<Self, String> {
        let path = normalize_path(path);
        let bytes = fs::read(&path).map_err(|err| format!("无法读取文件：{err}"))?;
        let metadata = fs::metadata(&path).map_err(|err| format!("无法读取文件信息：{err}"))?;
        let (encoding, text) = decode_text(&bytes);
        let raw_source = text.replace("\r\n", "\n");
        let (front, body, frontmatter_prefix) = split_front_matter(&raw_source);
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let (rendered, image_count, link_count) =
            render_markdown(&body, base_dir, ImageMode::Protocol);
        let (word_count, char_count) = count_text(&rendered);
        let title = front
            .title
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| first_h1(&body))
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "未命名文档".to_string())
            });
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        let modified_ms = modified
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_millis() as u64)
            .unwrap_or(0);
        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "未命名.md".to_string());
        let payload = DocumentPayload {
            path: path.to_string_lossy().to_string(),
            dir: base_dir.to_string_lossy().to_string(),
            file_name,
            title: title.clone(),
            html: rendered.clone(),
            raw: raw_source.clone(),
            word_count,
            char_count,
            read_minutes: ((word_count as f64 / 350.0).ceil() as usize).max(1),
            encoding,
            modified_ms,
            size: metadata.len(),
            image_count,
            link_count,
            front: front.clone(),
        };
        Ok(Self {
            path,
            title,
            payload,
            modified,
            source: body,
            raw_source,
            frontmatter_prefix,
        })
    }

    pub fn changed_on_disk(&self) -> bool {
        fs::metadata(&self.path)
            .and_then(|meta| meta.modified())
            .map(|modified| modified != self.modified)
            .unwrap_or(false)
    }
}

pub fn preview_html(source: &str, base_dir: &Path) -> (String, usize, usize) {
    let (_, body, _) = split_front_matter(source);
    let (html, _, _) = render_markdown(&body, base_dir, ImageMode::Protocol);
    let (words, chars) = count_text(&html);
    (html, words, chars)
}

/// Save edited Markdown. `body_only` is used by the WYSIWYG editor, which
/// does not touch the YAML front matter; the split editor saves the complete
/// source document.
pub fn save_markdown(
    doc: &LoadedDoc,
    markdown: &str,
    body_only: bool,
) -> Result<LoadedDoc, String> {
    let markdown = markdown.replace("\r\n", "\n");
    let mut output = String::new();
    if body_only {
        output.push_str(&doc.frontmatter_prefix);
    }
    output.push_str(&markdown);
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    fs::write(&doc.path, output.as_bytes()).map_err(|err| format!("无法写入文件：{err}"))?;
    LoadedDoc::load(&doc.path)
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let canonical = fs::canonicalize(&absolute).unwrap_or(absolute);
    let text = canonical.to_string_lossy();
    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        if stripped.len() >= 2 && stripped.as_bytes()[1] == b':' {
            return PathBuf::from(stripped);
        }
    }
    canonical
}

fn decode_text(bytes: &[u8]) -> (String, String) {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return (
            "UTF-8 (BOM)".to_string(),
            String::from_utf8_lossy(rest).into_owned(),
        );
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let (text, _, _) = encoding_rs::UTF_16LE.decode(rest);
        return ("UTF-16 LE".to_string(), text.into_owned());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let (text, _, _) = encoding_rs::UTF_16BE.decode(rest);
        return ("UTF-16 BE".to_string(), text.into_owned());
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => ("UTF-8".to_string(), text.to_string()),
        Err(_) => {
            let (text, _, had_errors) = encoding_rs::GB18030.decode(bytes);
            if !had_errors {
                ("GB18030".to_string(), text.into_owned())
            } else {
                (
                    "UTF-8 (含替换字符)".to_string(),
                    String::from_utf8_lossy(bytes).into_owned(),
                )
            }
        }
    }
}

fn split_front_matter(source: &str) -> (FrontMatter, String, String) {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let normalized = source.replace("\r\n", "\n");
    let mut lines = normalized.split('\n');
    if lines.next().map(str::trim) != Some("---") {
        return (FrontMatter::default(), normalized, String::new());
    }

    let mut front_lines = Vec::new();
    let mut consumed = 1usize;
    let mut found_end = false;
    for line in lines.by_ref() {
        consumed += 1;
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "..." {
            found_end = true;
            break;
        }
        front_lines.push(line.to_string());
    }
    if !found_end {
        return (FrontMatter::default(), normalized, String::new());
    }

    let mut front = FrontMatter::default();
    for line in front_lines {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().trim_matches(['"', '\'']).trim().to_string();
        match key.as_str() {
            "title" if !value.is_empty() => front.title = Some(value),
            "author" if !value.is_empty() => front.author = Some(value),
            "date" if !value.is_empty() => front.date = Some(value),
            "tags" if !value.is_empty() => {
                let cleaned = value
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|tag| tag.trim().trim_matches(['"', '\'']).to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect();
                front.tags = cleaned;
            }
            _ => {}
        }
    }

    let prefix: String = normalized.split_inclusive('\n').take(consumed).collect();
    let body: String = normalized.split_inclusive('\n').skip(consumed).collect();
    (front, body, prefix)
}

fn first_h1(source: &str) -> Option<String> {
    let mut in_fence = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let title = clean_inline_markdown(rest.trim());
            if !title.is_empty() {
                return Some(title);
            }
        }
    }
    None
}

fn clean_inline_markdown(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '`' | '*' | '_' | '~' => {}
            '[' => {
                // Keep the label, drop the destination of [label](url).
                let mut label = String::new();
                let mut depth = 1;
                for next in chars.by_ref() {
                    if next == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else if next == '[' {
                        depth += 1;
                    }
                    label.push(next);
                }
                if chars.peek() == Some(&'(') {
                    let mut paren = 0;
                    for next in chars.by_ref() {
                        if next == '(' {
                            paren += 1;
                        } else if next == ')' {
                            paren -= 1;
                            if paren == 0 {
                                break;
                            }
                        }
                    }
                }
                out.push_str(&label);
            }
            '<' => {
                // Drop simple inline HTML tags.
                let mut tag = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '>' {
                        closed = true;
                        break;
                    }
                    tag.push(next);
                }
                if !closed {
                    out.push('<');
                    out.push_str(&tag);
                }
            }
            _ => out.push(ch),
        }
    }
    out.trim().to_string()
}

fn render_markdown(source: &str, base_dir: &Path, mode: ImageMode) -> (String, usize, usize) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let mut image_count = 0usize;
    let mut link_count = 0usize;
    let parser = Parser::new_ext(source, options).map(|event| match event {
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            image_count += 1;
            let source = inline_image(&dest_url, base_dir, mode);
            Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::Boxed(source.into_boxed_str()),
                title,
                id,
            })
        }
        Event::Start(Tag::Link { .. }) => {
            link_count += 1;
            event
        }
        _ => event,
    });

    let mut output = String::with_capacity(source.len() + source.len() / 2);
    html::push_html(&mut output, parser);
    (output, image_count, link_count)
}

fn inline_image(source: &str, base_dir: &Path, mode: ImageMode) -> String {
    let source = source.trim();
    if source.is_empty() {
        return placeholder_image("空图片地址");
    }
    if source.starts_with("data:")
        || source.starts_with("http://")
        || source.starts_with("https://")
    {
        return source.to_string();
    }
    if source.starts_with("//") {
        return format!("https:{source}");
    }

    let path = local_image_path(source, base_dir);
    let Some(path) = path else {
        return placeholder_image("无法解析图片路径");
    };
    let Ok(metadata) = fs::metadata(&path) else {
        return placeholder_image(&format!(
            "图片未找到：{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        ));
    };
    // Keep the UI responsive when a document references enormous assets.
    if metadata.len() > 24 * 1024 * 1024 {
        return placeholder_image("图片超过 24 MB，已跳过");
    }
    if mode == ImageMode::Protocol {
        let encoded = utf8_percent_encode(&path.to_string_lossy(), QUERY_ENCODE_SET).to_string();
        return format!("/local?path={encoded}");
    }
    let Ok(bytes) = fs::read(&path) else {
        return placeholder_image("图片读取失败");
    };
    let mime = mime_for(&path);
    let encoded = BASE64.encode(bytes);
    format!("data:{mime};base64,{encoded}")
}

fn local_image_path(source: &str, base_dir: &Path) -> Option<PathBuf> {
    let source = source.split(['?', '#']).next().unwrap_or(source);
    if source.is_empty() {
        return None;
    }
    if let Some(rest) = source.strip_prefix("file://") {
        let decoded = percent_decode_str(rest).decode_utf8_lossy().to_string();
        if let Some(drive) = decoded.strip_prefix('/') {
            // file:///C:/dir/image.png -> C:/dir/image.png
            if drive.len() >= 2 && drive.as_bytes()[1] == b':' {
                return Some(PathBuf::from(drive.replace('/', "\\")));
            }
        }
        if decoded.starts_with("//") {
            // file://server/share/image.png -> \\server\share\image.png
            return Some(PathBuf::from(decoded.replace('/', "\\")));
        }
        if !decoded.starts_with('/') {
            return Some(PathBuf::from(format!("\\\\{}", decoded.replace('/', "\\"))));
        }
        return Some(PathBuf::from(decoded.replace('/', "\\")));
    }
    let decoded = percent_decode_str(source).decode_utf8_lossy().to_string();
    let path = PathBuf::from(decoded.replace('/', "\\"));
    if path.is_absolute() {
        Some(path)
    } else {
        Some(base_dir.join(path))
    }
}

pub fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}

fn placeholder_image(message: &str) -> String {
    let text = xml_escape(message);
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="560" height="120" viewBox="0 0 560 120"><rect width="560" height="120" rx="12" fill="#f1f3f5"/><path d="M36 60h36m-18-18v36" stroke="#adb5bd" stroke-width="4" stroke-linecap="round"/><text x="92" y="68" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="16" fill="#868e96">{text}</text></svg>"##
    );
    format!(
        "data:image/svg+xml;base64,{}",
        BASE64.encode(svg.as_bytes())
    )
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn count_text(html: &str) -> (usize, usize) {
    let mut text = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            in_tag = true;
            continue;
        }
        if ch == '>' && in_tag {
            in_tag = false;
            continue;
        }
        if in_tag {
            continue;
        }
        if ch == '&' {
            let mut entity = String::new();
            let mut closed = false;
            for next in chars.by_ref() {
                if next == ';' {
                    closed = true;
                    break;
                }
                if entity.len() > 8 {
                    break;
                }
                entity.push(next);
            }
            if closed {
                match entity.as_str() {
                    "amp" => text.push('&'),
                    "lt" => text.push('<'),
                    "gt" => text.push('>'),
                    "quot" => text.push('"'),
                    "nbsp" => text.push(' '),
                    _ => {}
                }
            }
            continue;
        }
        text.push(ch);
    }

    let mut words = 0usize;
    let mut in_word = false;
    let mut chars_count = 0usize;
    for ch in text.chars() {
        if ch.is_whitespace() {
            in_word = false;
            continue;
        }
        chars_count += 1;
        if is_cjk(ch) {
            words += 1;
            in_word = false;
        } else if ch.is_alphanumeric() {
            if !in_word {
                words += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }
    (words, chars_count)
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
    )
}

pub fn build_standalone_html(
    doc: &LoadedDoc,
    app_css: &str,
    highlight_js: &str,
    mermaid_js: &str,
    katex_js: &str,
    auto_render_js: &str,
    theme: &str,
) -> String {
    let base_dir = doc.path.parent().unwrap_or_else(|| Path::new("."));
    let (body, _, _) = render_markdown(&doc.source, base_dir, ImageMode::DataUri);
    let mut output = String::with_capacity(body.len() + app_css.len() + 260_000);
    output.push_str("<!doctype html>\n<html lang=\"zh-CN\" data-theme=\"");
    output.push_str(if theme == "dark" { "dark" } else { "light" });
    output.push_str("\">\n<head>\n<meta charset=\"utf-8\">\n");
    output.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    output.push_str("<meta name=\"generator\" content=\"mdview (Rust + WebView2)\">\n<title>");
    output.push_str(&html_escape(&doc.title));
    output.push_str("</title>\n<style>\n");
    output.push_str(app_css);
    output.push_str(EXPORT_CSS);
    output.push_str("</style>\n</head>\n<body class=\"export-page\">\n<main class=\"export-shell\">\n<header class=\"export-header\"><div class=\"export-kicker\">MARKDOWN</div><h1>");
    output.push_str(&html_escape(&doc.title));
    output.push_str("</h1><div class=\"export-meta\">");
    output.push_str(&html_escape(&doc.payload.file_name));
    output.push_str(" · ");
    output.push_str(&doc.payload.word_count.to_string());
    output.push_str(" 字 · 约 ");
    output.push_str(&doc.payload.read_minutes.to_string());
    output.push_str(" 分钟</div></header>\n<article class=\"markdown-body\" id=\"content\">");
    output.push_str(&body);
    output.push_str("</article>\n<footer class=\"export-footer\">由 mdview · Rust + WebView2 导出</footer>\n</main>\n");

    if body.contains("class=\"language-mermaid\"") && !mermaid_js.is_empty() {
        output.push_str("<script>");
        output.push_str(mermaid_js);
        output.push_str("</script>\n<script>mermaid.initialize({startOnLoad:false,securityLevel:'strict',theme:'");
        output.push_str(if theme == "dark" { "dark" } else { "default" });
        output.push_str("'});(async()=>{for(const code of [...document.querySelectorAll('code.language-mermaid')]){const pre=code.closest('pre');const box=document.createElement('div');box.className='mermaid';box.textContent=code.textContent;pre.replaceWith(box);}try{await mermaid.run({querySelector:'.mermaid'});}catch(e){console.error(e);}})();</script>\n");
    }

    if (body.contains('$') || body.contains("\\(") || body.contains("\\["))
        && !katex_js.is_empty()
        && !auto_render_js.is_empty()
    {
        output.push_str("<script>");
        output.push_str(katex_js);
        output.push_str("</script>\n<script>");
        output.push_str(auto_render_js);
        output.push_str("</script>\n<script>try{renderMathInElement(document.getElementById('content'),{delimiters:[{left:'$$',right:'$$',display:true},{left:'\\\\[',right:'\\\\]',display:true},{left:'$',right:'$',display:false},{left:'\\\\(',right:'\\\\)',display:false}],throwOnError:false,strict:false,output:'mathml',ignoredTags:['script','noscript','style','textarea','pre','code','option']});}catch(e){console.error(e);}</script>\n");
    }

    output.push_str("<script>");
    output.push_str(highlight_js);
    output.push_str("</script>\n<script>try{hljs.highlightAll();}catch(e){}</script>\n");
    output.push_str("</body>\n</html>\n");
    output
}

const EXPORT_CSS: &str = r#"
body.export-page { background: var(--bg); margin: 0; padding: 48px 20px 72px; }
.export-shell { width: min(860px, 100%); margin: 0 auto; background: var(--surface); border: 1px solid var(--border); border-radius: 18px; padding: 54px 64px 42px; box-shadow: var(--shadow); }
.export-header { padding-bottom: 26px; margin-bottom: 34px; border-bottom: 1px solid var(--border); }
.export-kicker { font-size: 11px; letter-spacing: .2em; color: var(--accent); font-weight: 700; margin-bottom: 10px; }
.export-header h1 { margin: 0 0 10px; font-size: 2.1rem; letter-spacing: -.03em; }
.export-meta { color: var(--muted); font-size: 13px; }
.export-footer { margin-top: 54px; padding-top: 18px; border-top: 1px solid var(--border); color: var(--muted); font-size: 12px; text-align: center; }
@media (max-width: 720px) { body.export-page { padding: 0; } .export-shell { border: 0; border-radius: 0; padding: 32px 22px; } }
@media print { body.export-page { padding: 0; background: #fff; } .export-shell { border: 0; box-shadow: none; border-radius: 0; width: 100%; padding: 0; } .export-header { margin-bottom: 24px; } .export-footer { display: none; } }
"#;

pub fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_title_drives_document_title() {
        let path = Path::new("examples/demo.md");
        let doc = LoadedDoc::load(path).expect("demo document should load");
        assert_eq!(doc.title, "mdview 功能演示");
        assert_eq!(doc.payload.encoding, "UTF-8");
        assert!(doc.payload.word_count > 100);
        assert!(doc.payload.html.contains("/local?path="));
        assert!(doc.raw_source.starts_with("---\n"));
        assert!(doc.frontmatter_prefix.ends_with("---\n"));
    }

    #[test]
    fn preview_strips_front_matter_and_renders_body() {
        let doc =
            LoadedDoc::load(Path::new("examples/demo.md")).expect("demo document should load");
        let (html, words, _) = preview_html(&doc.raw_source, Path::new("examples"));
        assert!(html.contains("mdview 功能演示"));
        assert!(!html.contains("tags: [markdown, rust, webview2]"));
        assert!(words > 100);
    }

    #[test]
    fn standalone_export_embeds_local_images() {
        let path = Path::new("examples/demo.md");
        let doc = LoadedDoc::load(path).expect("demo document should load");
        let html = build_standalone_html(
            &doc,
            crate::assets::APP_CSS,
            crate::assets::HIGHLIGHT_JS,
            crate::assets::MERMAID_JS,
            crate::assets::KATEX_JS,
            crate::assets::AUTO_RENDER_JS,
            "light",
        );
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("data:image/svg+xml;base64,"));
        assert!(!html.contains("/local?path="));
        assert!(html.contains("mdview 功能演示"));
        assert!(html.contains("renderMathInElement"));
    }

    #[test]
    fn inline_and_display_math_and_mermaid_are_rendered_as_markup() {
        let doc =
            LoadedDoc::load(Path::new("examples/demo.md")).expect("demo document should load");
        assert!(doc.payload.html.contains("class=\"language-mermaid\""));
        assert!(doc.payload.html.contains("x = \\frac"));
    }
}
