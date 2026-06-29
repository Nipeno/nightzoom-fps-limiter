// NightZoom FPS Limiter — installer backend.
//
// Commands the UI calls:
//   detect_fivem(manualPath?)             -> where FiveM is, whether ReShade exists, PC id,
//                                            and the version of the addon already installed (if any)
//   check_latest(installed?)              -> latest version available + the action to take
//                                            (install / update / reinstall)
//   install(fivemPath, replaceDxgi)       -> download the latest release, copy files, enable ReShade
//                                            (streams "progress" events to the UI)
//
// Update model is "download-mode": each run fetches the latest release from GitHub, so re-running
// the installer is how users update. The installed addon itself never phones home. For testing
// while the repo is still private, resolve_source() honours NZ_INSTALLER_SOURCE / NZ_GITHUB_TOKEN
// / NZ_INSTALLER_VERSION (see that function).
//
// The detection + Joaat id + INI write are a direct port of packaging/Enable-ReShade.bat.

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
    pub installed_version: Option<String>, // version of our addon already in plugins (None = not installed)
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
            // Read the file version off the already-installed addon (if any) so the UI can
            // decide install vs update. None when not installed or the version is unreadable.
            let installed_version = read_file_version(&plugins.join(ADDON_NAME));
            DetectResult {
                found: true,
                path: Some(dir.to_string_lossy().into_owned()),
                plugins: Some(plugins.to_string_lossy().into_owned()),
                existing_reshade: existing,
                installed_version,
                computer_name: name,
                reshade_id: id,
            }
        }
        None => DetectResult {
            found: false,
            path: None,
            plugins: None,
            existing_reshade: false,
            installed_version: None,
            computer_name: name,
            reshade_id: id,
        },
    }
}

/// Read the Windows FILEVERSION (as "major.minor.build") from a file's VERSIONINFO resource.
/// Returns None if the file is absent or carries no version resource. The addon stamps this via
/// CMake (see src/version.rc.in), so an addon built before versioning was added reads as None.
#[cfg(windows)]
fn read_file_version(path: &Path) -> Option<String> {
    use std::ffi::{c_void, OsStr};
    use std::os::windows::ffi::OsStrExt;

    if !path.exists() {
        return None;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    #[link(name = "version")]
    extern "system" {
        fn GetFileVersionInfoSizeW(filename: *const u16, handle: *mut u32) -> u32;
        fn GetFileVersionInfoW(filename: *const u16, handle: u32, len: u32, data: *mut c_void) -> i32;
        fn VerQueryValueW(block: *const c_void, sub: *const u16, buf: *mut *mut c_void, len: *mut u32) -> i32;
    }

    unsafe {
        let mut h: u32 = 0;
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), &mut h);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide.as_ptr(), 0, size, data.as_mut_ptr() as *mut c_void) == 0 {
            return None;
        }
        let sub: Vec<u16> = OsStr::new("\\").encode_wide().chain(std::iter::once(0)).collect();
        let mut buf: *mut c_void = std::ptr::null_mut();
        let mut len: u32 = 0;
        if VerQueryValueW(data.as_ptr() as *const c_void, sub.as_ptr(), &mut buf, &mut len) == 0
            || buf.is_null()
        {
            return None;
        }
        // VS_FIXEDFILEINFO: u32s at indices 2 (dwFileVersionMS) and 3 (dwFileVersionLS).
        let p = buf as *const u32;
        let ms = *p.add(2);
        let ls = *p.add(3);
        Some(format!("{}.{}.{}", ms >> 16, ms & 0xFFFF, ls >> 16))
    }
}

#[cfg(not(windows))]
fn read_file_version(_path: &Path) -> Option<String> {
    None
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

/// Resolve any candidate folder to the actual `FiveM.app` directory — the one that holds
/// `plugins\` and `CitizenFX.ini`. Handles the three shapes a user/registry hands us:
///   - they picked `FiveM.app` itself,
///   - they picked the parent that contains it (FiveM.exe sits in the parent on custom installs),
///   - a never-launched install where `CitizenFX.ini` / `plugins\` don't exist yet — we still
///     recognise `FiveM.app` by its folder name so detection works before first launch.
fn resolve_fivem_app(dir: &Path) -> Option<PathBuf> {
    let named_app = dir
        .file_name()
        .map(|n| n.eq_ignore_ascii_case("FiveM.app"))
        .unwrap_or(false);
    // Already the app folder (by name, or by the files a launched install leaves behind).
    // Must exist on disk — the default LOCALAPPDATA candidate is named FiveM.app but may not
    // be present (custom-path install, or FiveM not installed here at all).
    if dir.is_dir() && (named_app || dir.join("CitizenFX.ini").exists() || dir.join("plugins").is_dir()) {
        return Some(dir.to_path_buf());
    }
    // Parent that contains a FiveM.app subfolder (custom-path install; FiveM.exe lives here).
    let sub = dir.join("FiveM.app");
    if sub.is_dir() {
        return Some(sub);
    }
    None
}

fn find_fivem_app(manual: Option<&str>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(m) = manual {
        candidates.push(PathBuf::from(m));
    }

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local).join("FiveM").join("FiveM.app"));
    }

    // Custom install locations: the installer's Uninstall entry records InstallLocation,
    // and the fivem:// protocol handler points at FiveM.exe (once registered).
    candidates.extend(fivem_from_uninstall());
    if let Some(dir) = fivem_from_registry() {
        candidates.push(dir);
    }

    candidates.into_iter().find_map(|d| resolve_fivem_app(&d))
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

/// Scan the Windows "Uninstall" registry (HKCU + HKLM, incl. WOW6432Node) for FiveM's
/// entry and return its install directory. This is how custom install paths are recoverable
/// without the user browsing — the installer writes InstallLocation / DisplayIcon there.
#[cfg(windows)]
fn fivem_from_uninstall() -> Vec<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    const ROOTS: [(isize, &str); 3] = [
        (HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    let mut out: Vec<PathBuf> = Vec::new();
    for (root, path) in ROOTS {
        let hk = RegKey::predef(root);
        let unins = match hk.open_subkey(path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub in unins.enum_keys().flatten() {
            let app = match unins.open_subkey(&sub) {
                Ok(k) => k,
                Err(_) => continue,
            };
            let name: String = app.get_value("DisplayName").unwrap_or_default();
            if !name.to_lowercase().contains("fivem") {
                continue;
            }
            // InstallLocation is the FiveM root dir; DisplayIcon is the FiveM.exe path.
            if let Ok(loc) = app.get_value::<String, _>("InstallLocation") {
                let p = PathBuf::from(loc.trim().trim_matches('"'));
                if !p.as_os_str().is_empty() {
                    out.push(p);
                }
            }
            if let Ok(icon) = app.get_value::<String, _>("DisplayIcon") {
                if let Some(dir) = parse_exe_dir(&icon) {
                    out.push(dir);
                }
            }
        }
    }
    out
}

#[cfg(not(windows))]
fn fivem_from_uninstall() -> Vec<PathBuf> {
    Vec::new()
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

    // 1) Get the latest release zip (GitHub download-mode, or a dev override source).
    prog(app, "download", "start", 0.0, "Getting the latest version…");
    let resolved = resolve_source().map_err(|e| {
        prog(app, "download", "fail", 0.0, &e);
        e
    })?;
    let zip_bytes = fetch_zip(app, &resolved.fetch).map_err(|e| {
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

// ---- Version check (check_latest command) ---------------------------------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LatestInfo {
    version: String,
    action: String, // "install" | "update" | "reinstall"
}

/// What the latest release is, and what to do given what's installed. The UI calls this after
/// detect_fivem and passes the detected installed version so the comparison happens in Rust
/// (a JS string compare gets 1.10 vs 1.9 wrong).
#[tauri::command]
async fn check_latest(installed: Option<String>) -> Result<LatestInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let resolved = resolve_source()?;
        let action = decide_action(installed.as_deref(), &resolved.version);
        Ok(LatestInfo { version: resolved.version, action })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// install (none present) / update (newer available) / reinstall (same or older available).
/// Unparseable versions fall back to "update" so the user is never stuck on a stale build.
fn decide_action(installed: Option<&str>, latest: &str) -> String {
    use semver::Version;
    match installed {
        None => "install".to_string(),
        Some(cur) => {
            let cur = cur.trim().trim_start_matches(['v', 'V']);
            match (Version::parse(cur), Version::parse(latest)) {
                (Ok(c), Ok(l)) if l > c => "update".to_string(),
                (Ok(_), Ok(_)) => "reinstall".to_string(),
                _ => "update".to_string(),
            }
        }
    }
}

// ---- Source resolution + download -----------------------------------------

/// Where to fetch the release zip from, and the version it represents.
struct Resolved {
    version: String,
    fetch: Fetch,
}

enum Fetch {
    LocalFile(PathBuf),                       // a zip on disk (dev override)
    Http { url: String, token: Option<String> }, // GitHub asset / direct URL
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn strip_v(s: &str) -> String {
    s.trim().trim_start_matches(['v', 'V']).to_string()
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("nz-fps-limiter-installer")
        .build()
        .map_err(|e| e.to_string())
}

/// Decide where the release zip comes from.
///
/// Testing while the repo is private (none of these are needed in production):
///   - NZ_INSTALLER_SOURCE  = a local .zip path OR an http(s) URL — bypasses GitHub entirely,
///                            so the full install flow is testable with no repo access.
///   - NZ_INSTALLER_VERSION = the version that override should report as "latest" (default 0.0.0).
///   - NZ_GITHUB_TOKEN      = a PAT with `repo` scope — lets the normal GitHub path read the
///                            private repo's latest release and download its (private) asset.
fn resolve_source() -> Result<Resolved, String> {
    if let Some(src) = env_nonempty("NZ_INSTALLER_SOURCE") {
        let version = env_nonempty("NZ_INSTALLER_VERSION")
            .map(|s| strip_v(&s))
            .unwrap_or_else(|| "0.0.0".to_string());
        let fetch = if src.starts_with("http://") || src.starts_with("https://") {
            Fetch::Http { url: src, token: None }
        } else {
            Fetch::LocalFile(PathBuf::from(src))
        };
        return Ok(Resolved { version, fetch });
    }
    resolve_github()
}

fn resolve_github() -> Result<Resolved, String> {
    let token = env_nonempty("NZ_GITHUB_TOKEN");
    let client = http_client()?;

    let api = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let mut req = client.get(&api).header("Accept", "application/vnd.github+json");
    if let Some(t) = &token {
        req = req.bearer_auth(t);
    }
    let resp = req
        .send()
        .map_err(|e| format!("Couldn't reach GitHub: {e}"))?
        .error_for_status()
        .map_err(|e| {
            if e.status().map(|s| s.as_u16()) == Some(404) && token.is_none() {
                "Couldn't find a release. If the repo is still private, set NZ_GITHUB_TOKEN to a \
                 PAT with repo access.".to_string()
            } else {
                format!("GitHub returned an error: {e}")
            }
        })?;
    let json: serde_json::Value = resp
        .json()
        .map_err(|e| format!("Bad response from GitHub: {e}"))?;

    let version = json["tag_name"]
        .as_str()
        .map(strip_v)
        .ok_or_else(|| "Latest release has no tag.".to_string())?;

    // Asset-name contract with .github/workflows/build.yml: `NZ-FPS-Limiter_v<version>.zip`.
    // On a private repo the public browser_download_url needs auth, so when we have a token we
    // hit the asset API endpoint (a["url"]) with Accept: application/octet-stream instead.
    let url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|a| {
                let name = a["name"].as_str()?;
                if name.starts_with("NZ-FPS-Limiter_v") && name.ends_with(".zip") {
                    let key = if token.is_some() { "url" } else { "browser_download_url" };
                    a[key].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| "No installer zip found in the latest release.".to_string())?;

    Ok(Resolved { version, fetch: Fetch::Http { url, token } })
}

fn fetch_zip(app: &AppHandle, fetch: &Fetch) -> Result<Vec<u8>, String> {
    match fetch {
        Fetch::LocalFile(p) => {
            prog(app, "download", "start", -1.0, "Loading bundled files…");
            std::fs::read(p).map_err(|e| format!("Couldn't read {}: {e}", p.display()))
        }
        Fetch::Http { url, token } => download_http(app, url, token.as_deref()),
    }
}

fn download_http(app: &AppHandle, url: &str, token: Option<&str>) -> Result<Vec<u8>, String> {
    let client = http_client()?;
    let mut req = client.get(url);
    if let Some(t) = token {
        // Private-repo asset endpoint: needs auth + octet-stream to return the binary.
        req = req.bearer_auth(t).header("Accept", "application/octet-stream");
    }
    let mut resp = req
        .send()
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download failed: {e}"))?;

    // Stream the download so we can report percentage.
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
        .invoke_handler(tauri::generate_handler![detect_fivem, check_latest, install, relaunch_as_admin])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
