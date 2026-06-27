// NightZoom FPS Limiter — installer backend.
//
// Two commands the UI calls:
//   detect_fivem(manualPath?)             -> where FiveM is, whether ReShade exists, PC id
//   install(fivemPath, replaceDxgi)       -> download latest release, copy files, enable ReShade
//                                            (streams "progress" events to the UI)
//
// This is a direct port of packaging/Enable-ReShade.bat (detection + Joaat id + INI write),
// plus the file download/copy the bat never did.

use std::path::{Path, PathBuf};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const REPO: &str = "Nipeno/nightzoom-fps-limiter";
const ADDON_NAME: &str = "NZ-FPS-Limiter.addon64";
const ACK: &str =
    "acknowledged that ReShade 5.x has a bug that will lead to game crashes";

// ---------------------------------------------------------------------------
// detect_fivem
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DetectResult {
    pub found: bool,
    pub path: Option<String>,        // FiveM.app folder
    pub plugins: Option<String>,     // FiveM.app\plugins
    pub existing_reshade: bool,      // plugins\dxgi.dll already present (graphics pack)
    pub computer_name: String,
    pub reshade_id: String,
}

#[tauri::command]
fn detect_fivem(manual_path: Option<String>) -> DetectResult {
    let name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "a".into());
    let id = reshade_id(&name);

    let app = find_fivem_app(manual_path.as_deref());
    match app {
        Some(dir) => {
            let plugins = dir.join("plugins");
            let existing = plugins.join("dxgi.dll").exists();
            DetectResult {
                found: true,
                path: Some(dir.to_string_lossy().into_owned()),
                plugins: Some(plugins.to_string_lossy().into_owned()),
                existing_reshade: existing,
                computer_name: name,
                reshade_id: id,
            }
        }
        None => DetectResult {
            found: false,
            path: None,
            plugins: None,
            existing_reshade: false,
            computer_name: name,
            reshade_id: id,
        },
    }
}

/// FiveM's ReShade5 acknowledgement id = Joaat(lowercase(name)), ASCII only.
/// Verified against the bat: PC -> 46750aa6.
fn reshade_id(name: &str) -> String {
    let mut h: u32 = 0;
    for &b in name.as_bytes() {
        let mut c = b as u32;
        if (b'A'..=b'Z').contains(&b) {
            c += 32; // ASCII lower
        }
        h = h.wrapping_add(c);
        h = h.wrapping_add(h << 10);
        h ^= h >> 6;
    }
    h = h.wrapping_add(h << 3);
    h ^= h >> 11;
    h = h.wrapping_add(h << 15);
    format!("{:08x}", h)
}

fn is_fivem_app(dir: &Path) -> bool {
    dir.join("FiveM.exe").exists()
        || dir.join("CitizenFX.ini").exists()
        || dir.join("plugins").exists()
}

fn find_fivem_app(manual: Option<&str>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(m) = manual {
        let p = PathBuf::from(m);
        // Accept either the FiveM.app folder itself or its parent (e.g. user picked FiveM\).
        candidates.push(p.clone());
        candidates.push(p.join("FiveM.app"));
    }

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local).join("FiveM").join("FiveM.app"));
    }

    if let Some(dir) = fivem_from_registry() {
        candidates.push(dir);
    }

    candidates.into_iter().find(|d| is_fivem_app(d))
}

/// Read the fivem:// protocol handler and pull the FiveM.exe directory out of it.
#[cfg(windows)]
fn fivem_from_registry() -> Option<PathBuf> {
    use winreg::enums::HKEY_CLASSES_ROOT;
    use winreg::RegKey;
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let key = hkcr.open_subkey(r"fivem\shell\open\command").ok()?;
    let cmd: String = key.get_value("").ok()?;
    parse_exe_dir(&cmd)
}

#[cfg(not(windows))]
fn fivem_from_registry() -> Option<PathBuf> {
    None
}

fn parse_exe_dir(cmd: &str) -> Option<PathBuf> {
    let s = cmd.trim();
    let path = if let Some(rest) = s.strip_prefix('"') {
        rest.split('"').next()?.to_string()
    } else {
        let idx = s.to_lowercase().find(".exe")? + 4;
        s[..idx].to_string()
    };
    PathBuf::from(path).parent().map(|p| p.to_path_buf())
}

// ---------------------------------------------------------------------------
// install
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
struct Progress {
    step: String,    // download | copy | enable | done
    status: String,  // start | ok | fail
    percent: f64,    // 0..100 (download only; -1 = indeterminate)
    message: String,
}

fn prog(app: &AppHandle, step: &str, status: &str, percent: f64, message: &str) {
    let _ = app.emit(
        "progress",
        Progress {
            step: step.into(),
            status: status.into(),
            percent,
            message: message.into(),
        },
    );
}

#[tauri::command]
async fn install(app: AppHandle, fivem_path: String, replace_dxgi: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || install_blocking(&app, &fivem_path, replace_dxgi))
        .await
        .map_err(|e| e.to_string())?
}

fn install_blocking(app: &AppHandle, fivem_path: &str, replace_dxgi: bool) -> Result<(), String> {
    let plugins = PathBuf::from(fivem_path).join("plugins");
    std::fs::create_dir_all(&plugins).map_err(|e| format!("Can't create plugins folder: {e}"))?;

    // 1) Download the latest release zip from GitHub.
    prog(app, "download", "start", 0.0, "Downloading the latest version…");
    let zip_bytes = download_latest_zip(app).map_err(|e| {
        prog(app, "download", "fail", 0.0, &e);
        e
    })?;
    prog(app, "download", "ok", 100.0, "Download complete");

    // 2) Pull dxgi.dll + the addon out of the zip.
    let (dxgi, addon) = extract_payload(&zip_bytes).map_err(|e| {
        prog(app, "copy", "fail", 0.0, &e);
        e
    })?;

    // 3) Copy into FiveM\plugins.
    prog(app, "copy", "start", 0.0, "Copying files into FiveM…");
    let addon_dst = plugins.join(ADDON_NAME);
    write_file(&addon_dst, &addon).map_err(|e| {
        let m = locked_hint(&format!("Couldn't write {}: {e}", addon_dst.display()));
        prog(app, "copy", "fail", 0.0, &m);
        m
    })?;

    let dxgi_dst = plugins.join("dxgi.dll");
    if replace_dxgi || !dxgi_dst.exists() {
        write_file(&dxgi_dst, &dxgi).map_err(|e| {
            let m = locked_hint(&format!("Couldn't write {}: {e}", dxgi_dst.display()));
            prog(app, "copy", "fail", 0.0, &m);
            m
        })?;
    }
    prog(app, "copy", "ok", 100.0, "Files copied");

    // 4) Enable ReShade in CitizenFX.ini (Joaat id + INI write, same as the bat).
    prog(app, "enable", "start", 0.0, "Enabling ReShade in FiveM…");
    let name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "a".into());
    let value = format!("ID:{} {}", reshade_id(&name), ACK);
    let ini = PathBuf::from(fivem_path).join("CitizenFX.ini");
    enable_reshade(&ini, &value).map_err(|e| {
        let m = locked_hint(&e);
        prog(app, "enable", "fail", 0.0, &m);
        m
    })?;
    prog(app, "enable", "ok", 100.0, "ReShade enabled");

    prog(app, "done", "ok", 100.0, "All set");
    Ok(())
}

fn locked_hint(msg: &str) -> String {
    format!("{msg}\n\nClose FiveM if it's open, or run the installer as administrator, then retry.")
}

fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, bytes)
}

// ---- GitHub download ------------------------------------------------------

fn download_latest_zip(app: &AppHandle) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("nz-fps-limiter-installer")
        .build()
        .map_err(|e| e.to_string())?;

    let api = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let json: serde_json::Value = client
        .get(&api)
        .send()
        .map_err(|e| format!("Couldn't reach GitHub: {e}"))?
        .error_for_status()
        .map_err(|e| format!("GitHub returned an error: {e}"))?
        .json()
        .map_err(|e| format!("Bad response from GitHub: {e}"))?;

    // Asset-name contract with .github/workflows/build.yml: it ships the bundle as
    // `NZ-FPS-Limiter_v<version>.zip`. If that name ever changes, update this match.
    let url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|a| {
                let name = a["name"].as_str()?;
                if name.starts_with("NZ-FPS-Limiter_v") && name.ends_with(".zip") {
                    a["browser_download_url"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| "No installer zip found in the latest release.".to_string())?;

    // Stream the download so we can report percentage.
    let mut resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download failed: {e}"))?;

    let total = resp.content_length().unwrap_or(0);
    let mut buf: Vec<u8> = Vec::with_capacity(total as usize);
    let mut chunk = [0u8; 64 * 1024];
    use std::io::Read;
    loop {
        let n = resp.read(&mut chunk).map_err(|e| format!("Download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        let pct = if total > 0 {
            (buf.len() as f64 / total as f64) * 100.0
        } else {
            -1.0
        };
        prog(app, "download", "start", pct, "Downloading the latest version…");
    }
    Ok(buf)
}

/// Find dxgi.dll and the .addon64 anywhere in the zip (they live under
/// Copy-these-into-plugins/ in the release, but match by name to be safe).
fn extract_payload(zip_bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    use std::io::Read;
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("Bad zip: {e}"))?;

    let mut dxgi: Option<Vec<u8>> = None;
    let mut addon: Option<Vec<u8>> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_lowercase();
        if name.ends_with("dxgi.dll") {
            let mut v = Vec::new();
            file.read_to_end(&mut v).map_err(|e| e.to_string())?;
            dxgi = Some(v);
        } else if name.ends_with(".addon64") {
            let mut v = Vec::new();
            file.read_to_end(&mut v).map_err(|e| e.to_string())?;
            addon = Some(v);
        }
    }

    match (dxgi, addon) {
        (Some(d), Some(a)) => Ok((d, a)),
        _ => Err("The release zip is missing dxgi.dll or the addon.".into()),
    }
}

// ---- CitizenFX.ini edit (Win32 INI API, same as the bat) ------------------

#[cfg(windows)]
fn enable_reshade(ini: &Path, value: &str) -> Result<(), String> {
    let path = ini.to_string_lossy().to_string();
    let current = ini_read("Addons", "ReShade5", &path);
    if current == value {
        return Ok(()); // already enabled
    }
    if ini_write("Addons", "ReShade5", value, &path) {
        Ok(())
    } else {
        Err(format!("Couldn't write {}.", path))
    }
}

#[cfg(not(windows))]
fn enable_reshade(_ini: &Path, _value: &str) -> Result<(), String> {
    Err("Windows only.".into())
}

#[cfg(windows)]
mod winini {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn WritePrivateProfileStringW(
            section: *const u16,
            key: *const u16,
            value: *const u16,
            file: *const u16,
        ) -> i32;
        pub fn GetPrivateProfileStringW(
            section: *const u16,
            key: *const u16,
            default: *const u16,
            returned: *mut u16,
            size: u32,
            file: *const u16,
        ) -> u32;
    }

    pub fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }
}

#[cfg(windows)]
fn ini_read(section: &str, key: &str, file: &str) -> String {
    use winini::*;
    let s = wide(section);
    let k = wide(key);
    let def = wide("");
    let f = wide(file);
    let mut buf = vec![0u16; 1024];
    let n = unsafe {
        GetPrivateProfileStringW(
            s.as_ptr(),
            k.as_ptr(),
            def.as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            f.as_ptr(),
        )
    };
    String::from_utf16_lossy(&buf[..n as usize])
}

#[cfg(windows)]
fn ini_write(section: &str, key: &str, value: &str, file: &str) -> bool {
    use winini::*;
    let s = wide(section);
    let k = wide(key);
    let v = wide(value);
    let f = wide(file);
    unsafe { WritePrivateProfileStringW(s.as_ptr(), k.as_ptr(), v.as_ptr(), f.as_ptr()) != 0 }
}

#[cfg(not(windows))]
fn ini_read(_s: &str, _k: &str, _f: &str) -> String {
    String::new()
}
#[cfg(not(windows))]
fn ini_write(_s: &str, _k: &str, _v: &str, _f: &str) -> bool {
    false
}

// ---------------------------------------------------------------------------

/// Relaunch this installer elevated (UAC "Run as administrator"), then close.
/// Used when a file write was blocked by permissions.
#[tauri::command]
fn relaunch_as_admin(app: AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let wide: Vec<u16> = std::ffi::OsStr::new(exe.as_os_str())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let verb: Vec<u16> = std::ffi::OsStr::new("runas")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        #[link(name = "shell32")]
        extern "system" {
            fn ShellExecuteW(
                hwnd: isize,
                lpoperation: *const u16,
                lpfile: *const u16,
                lpparameters: *const u16,
                lpdirectory: *const u16,
                nshowcmd: i32,
            ) -> isize;
        }
        let r = unsafe {
            ShellExecuteW(0, verb.as_ptr(), wide.as_ptr(), std::ptr::null(), std::ptr::null(), 1)
        };
        if r <= 32 {
            return Err("Couldn't relaunch as administrator.".into());
        }
        app.exit(0);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("Windows only.".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![detect_fivem, install, relaunch_as_admin])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
