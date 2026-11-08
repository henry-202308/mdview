#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

const PROG_ID: &str = "mdview.md";
const APP_NAME: &str = "mdview";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn run_reg(args: &[&str]) -> Result<(), String> {
    let mut command = Command::new("reg.exe");
    command.args(args);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .map_err(|err| format!("无法运行 reg.exe：{err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = decode_console(&output.stderr);
        let stdout = decode_console(&output.stdout);
        Err(if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        })
    }
}

fn decode_console(bytes: &[u8]) -> String {
    let (text, _, _) = encoding_rs::GBK.decode(bytes);
    text.into_owned()
}

pub fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|err| format!("无法获取程序路径：{err}"))?;
    let exe = exe.to_string_lossy().to_string();
    let command = format!("\"{exe}\" \"%1\"");
    let icon = format!("\"{exe}\",0");

    run_reg(&[
        "add",
        r"HKCU\Software\Classes\mdview.md",
        "/ve",
        "/d",
        "Markdown 文档",
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\mdview.md\DefaultIcon",
        "/ve",
        "/d",
        &icon,
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\mdview.md\shell\open",
        "/ve",
        "/d",
        "打开(&O)",
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\mdview.md\shell\open\command",
        "/ve",
        "/d",
        &command,
        "/f",
    ])?;

    // Make this the default handler for .md for the current user. This is
    // exactly what the "关联 .md 文件" action promises, and it only touches
    // HKCU, so no administrator rights are required.
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\.md",
        "/ve",
        "/d",
        PROG_ID,
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\.md",
        "/v",
        "Content Type",
        "/d",
        "text/markdown",
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\.md",
        "/v",
        "PerceivedType",
        "/d",
        "text",
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\.md\OpenWithProgids",
        "/v",
        PROG_ID,
        "/t",
        "REG_NONE",
        "/d",
        "",
        "/f",
    ])?;

    run_reg(&[
        "add",
        r"HKCU\Software\Classes\Applications\mdview.exe\shell\open\command",
        "/ve",
        "/d",
        &command,
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\Applications\mdview.exe",
        "/v",
        "FriendlyAppName",
        "/d",
        APP_NAME,
        "/f",
    ])?;
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    // Only delete our own ProgID and our OpenWith entry. Never restore or
    // remove another application's association behind the user's back.
    run_reg(&["delete", r"HKCU\Software\Classes\mdview.md", "/f"]).ok();
    run_reg(&[
        "delete",
        r"HKCU\Software\Classes\Applications\mdview.exe",
        "/f",
    ])
    .ok();
    run_reg(&[
        "delete",
        r"HKCU\Software\Classes\.md\OpenWithProgids",
        "/v",
        PROG_ID,
        "/f",
    ])
    .ok();

    let current = query_default(r"HKCU\Software\Classes\.md");
    if current
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case(PROG_ID))
        .unwrap_or(false)
    {
        run_reg(&["delete", r"HKCU\Software\Classes\.md", "/ve", "/f"])
            .map_err(|err| format!("无法解除 .md 默认关联：{err}"))?;
    }
    Ok(())
}

fn query_default(key: &str) -> Option<String> {
    let mut command = Command::new("reg.exe");
    command.args(["query", key, "/ve"]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().find(|line| line.contains("REG_SZ"))?;
    let marker = "REG_SZ";
    let index = line.find(marker)? + marker.len();
    Some(line[index..].trim().to_string())
}
