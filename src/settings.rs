use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const ACCENTS: [&str; 8] = [
    "github", "xhs", "tech", "girl", "colorful", "duolingo", "linear", "notion",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// "system", "light" or "dark"
    pub theme: String,
    /// One of [`ACCENTS`].
    pub accent: String,
    pub font_size: u32,
    pub reading_width: u32,
    pub toc: bool,
    pub font: String,
    pub recent: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            accent: "github".to_string(),
            font_size: 16,
            reading_width: 760,
            toc: true,
            font: "sans".to_string(),
            recent: Vec::new(),
        }
    }
}

impl Settings {
    pub fn sanitize(&mut self) {
        if !matches!(self.theme.as_str(), "system" | "light" | "dark") {
            self.theme = "system".to_string();
        }
        if !ACCENTS.contains(&self.accent.as_str()) {
            self.accent = "github".to_string();
        }
        self.font_size = self.font_size.clamp(13, 24);
        self.reading_width = self.reading_width.clamp(600, 980);
        if !matches!(self.font.as_str(), "sans" | "serif") {
            self.font = "sans".to_string();
        }
        self.recent.truncate(12);
    }

    pub fn record_recent(&mut self, path: &Path) {
        let value = path.to_string_lossy().to_string();
        self.recent
            .retain(|item| !item.eq_ignore_ascii_case(&value));
        self.recent.insert(0, value);
        self.recent.truncate(10);
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let path = PathBuf::from(appdata).join("mdview");
        if fs::create_dir_all(&path).is_ok() {
            return path;
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let path = PathBuf::from(local).join("mdview");
        if fs::create_dir_all(&path).is_ok() {
            return path;
        }
    }
    // Portable / sandboxed fallback: keep preferences next to the executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let path = parent.join(".mdview-data");
            if fs::create_dir_all(&path).is_ok() {
                return path;
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_path();
        let mut settings = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
            .unwrap_or_default();
        settings.sanitize();
        settings
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = config_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("settings.json");
        let text = serde_json::to_string_pretty(self)?;
        // Write next to the destination first so a crash cannot leave a
        // truncated settings file behind.
        let tmp = dir.join("settings.json.tmp");
        fs::write(&tmp, text)?;
        if fs::rename(&tmp, &path).is_err() {
            // Windows does not replace an existing file with rename().
            let _ = fs::remove_file(&path);
            fs::rename(&tmp, &path)?;
        }
        Ok(())
    }
}
