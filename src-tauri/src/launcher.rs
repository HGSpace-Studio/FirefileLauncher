use futures_util::StreamExt;
use minecraft_java_rs_core::launcher::events::LaunchEvent;
use minecraft_java_rs_core::launcher::options::{
    JavaOptions, LaunchOptions, LoaderConfig, MemoryConfig, ScreenConfig,
};
use minecraft_java_rs_core::launcher::Launcher;
use minecraft_java_rs_core::models::loader::LoaderType;
use minecraft_java_rs_core::models::minecraft::Authenticator;
use minecraft_java_rs_core::utils::auth::offline_uuid;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

pub type GameStopSignal = Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>;
pub type GameProcessState = Arc<Mutex<Option<u32>>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CrashReport {
    pub instance_name: String,
    pub game_version: String,
    pub java_version: String,
    pub system_version: String,
    pub error_message: String,
    pub crash_log: String,
    pub solution: String,
}

pub type CrashReportState = Arc<Mutex<Option<CrashReport>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricVersion {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeBuild {
    #[serde(rename = "_id")]
    pub id: String,
    pub build: u64,
    pub version: String,
    pub mcversion: String,
    pub modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaInstall {
    pub path: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMemory {
    pub total_mb: u64,
    pub used_mb: u64,
}

#[tauri::command]
pub async fn get_system_memory() -> Result<SystemMemory, String> {
    let mem = tokio::task::spawn_blocking(|| {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        (
            sys.total_memory() / 1024 / 1024,
            sys.used_memory() / 1024 / 1024,
        )
    })
    .await
    .map_err(|e| format!("{e}"))?;
    Ok(SystemMemory {
        total_mb: mem.0,
        used_mb: mem.1,
    })
}

#[tauri::command]
pub async fn get_java_versions() -> Result<Vec<JavaInstall>, String> {
    let results = tokio::task::spawn_blocking(discover_java)
        .await
        .map_err(|e| format!("Java discovery task failed: {e}"))?;
    Ok(results)
}

fn discover_java() -> Vec<JavaInstall> {
    let mut found = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. Check JAVA_HOME
    if let Ok(jh) = std::env::var("JAVA_HOME") {
        let path = std::path::Path::new(&jh);
        let java_bin = if cfg!(target_os = "windows") {
            path.join("bin").join("java.exe")
        } else {
            path.join("bin").join("java")
        };
        if java_bin.exists() {
            if let Some(info) = probe_java(&java_bin) {
                if seen.insert(info.path.clone()) {
                    found.push(info);
                }
            }
        }
    }

    // 2. macOS: /usr/libexec/java_home -V
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/libexec/java_home")
            .arg("-V")
            .output();
        if let Ok(out) = output {
            let stderr = String::from_utf8_lossy(&out.stderr);
            for line in stderr.lines() {
                if let Some(idx) = line.find('/') {
                    let path_part = &line[idx..].trim();
                    let java_bin = std::path::Path::new(path_part).join("bin").join("java");
                    if java_bin.exists() {
                        if let Some(info) = probe_java(&java_bin) {
                            if seen.insert(info.path.clone()) {
                                found.push(info);
                            }
                        }
                    }
                }
            }
        }
        // Also scan /Library/Java/JavaVirtualMachines/
        if let Ok(entries) = std::fs::read_dir("/Library/Java/JavaVirtualMachines/") {
            for entry in entries.flatten() {
                let java_bin = entry
                    .path()
                    .join("Contents")
                    .join("Home")
                    .join("bin")
                    .join("java");
                if java_bin.exists() {
                    if let Some(info) = probe_java(&java_bin) {
                        if seen.insert(info.path.clone()) {
                            found.push(info);
                        }
                    }
                }
            }
        }
    }

    // 3. Linux: /usr/lib/jvm/
    #[cfg(target_os = "linux")]
    {
        let jvm_base = std::path::Path::new("/usr/lib/jvm");
        if jvm_base.exists() {
            if let Ok(entries) = std::fs::read_dir(jvm_base) {
                for entry in entries.flatten() {
                    let java_bin = entry.path().join("bin").join("java");
                    if java_bin.exists() {
                        if let Some(info) = probe_java(&java_bin) {
                            if seen.insert(info.path.clone()) {
                                found.push(info);
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Windows: common install paths
    #[cfg(target_os = "windows")]
    {
        let prog_files = ["Program Files", "Program Files (x86)"];
        for pf in &prog_files {
            let path = std::path::Path::new("C:\\").join(pf).join("Java");
            if path.exists() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let java_bin = entry.path().join("bin").join("java.exe");
                        if java_bin.exists() {
                            if let Some(info) = probe_java(&java_bin) {
                                if seen.insert(info.path.clone()) {
                                    found.push(info);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Also try `where java`
        if let Ok(out) = std::process::Command::new("where").arg("java").output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let p = line.trim().to_string();
                if !p.is_empty() && seen.insert(p.clone()) {
                    let java_bin = std::path::Path::new(&p);
                    if let Some(info) = probe_java(java_bin) {
                        found.push(info);
                    }
                }
            }
        }
    }

    // 5. Check PATH for `java`
    let java_cmd = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let java_bin = dir.join(java_cmd);
            if java_bin.exists() {
                if let Some(info) = probe_java(&java_bin) {
                    if seen.insert(info.path.clone()) {
                        found.push(info);
                    }
                }
            }
        }
    }

    found
}

fn probe_java(path: &std::path::Path) -> Option<JavaInstall> {
    let path_str = path.to_string_lossy().to_string();
    let output = std::process::Command::new(path)
        .arg("-version")
        .output()
        .ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = stderr.lines().next().unwrap_or("").trim().to_string();
    Some(JavaInstall {
        path: path_str,
        version,
    })
}

#[tauri::command]
pub async fn get_fabric_versions() -> Result<Vec<FabricVersion>, String> {
    let url = "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader";
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch fabric versions: {e}"))?;
    let versions: Vec<FabricVersion> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse fabric versions: {e}"))?;
    Ok(versions)
}

#[tauri::command]
pub async fn get_forge_versions(mc_version: String) -> Result<Vec<ForgeBuild>, String> {
    let url = format!(
        "https://bmclapi2.bangbang93.com/forge/minecraft/{}",
        mc_version
    );
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch forge versions: {e}"))?;
    let builds: Vec<ForgeBuild> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse forge versions: {e}"))?;
    Ok(builds)
}

#[tauri::command]
pub async fn get_minecraft_versions() -> Result<VersionManifest, String> {
    let url = "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json";
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch versions: {e}"))?;
    let manifest: VersionManifest = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse versions: {e}"))?;
    Ok(manifest)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchArgs {
    pub version: String,
    pub username: String,
    pub game_dir: String,
    pub min_mem: String,
    pub max_mem: String,
    pub java_path: Option<String>,
    pub loader_type: Option<String>,
    pub loader_build: Option<String>,
    pub instance: Option<String>,
    pub download_only: bool,
    pub download_concurrency: Option<u32>,
    pub verify_concurrency: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MinecraftEvent {
    Progress {
        kind: String,
        downloaded: u64,
        total: u64,
    },
    Data {
        line: String,
    },
    Close {
        code: i32,
    },
    Error {
        message: String,
    },
    GameDownloadFinished,
}

#[tauri::command]
pub async fn launch_minecraft(app: AppHandle, args: LaunchArgs) -> Result<(), String> {
    let auth = Authenticator {
        access_token: "offline".into(),
        name: args.username.clone(),
        uuid: offline_uuid(&args.username),
        xbox_account: None,
        user_properties: None,
        client_id: None,
        client_token: None,
    };

    let loader = if let Some(loader_type_str) = args.loader_type {
        let loader_type = match loader_type_str.to_lowercase().as_str() {
            "forge" => LoaderType::Forge,
            "neoforge" | "neo_forge" => LoaderType::NeoForge,
            "fabric" => LoaderType::Fabric,
            "legacyfabric" | "legacy_fabric" => LoaderType::LegacyFabric,
            "quilt" => LoaderType::Quilt,
            _ => return Err(format!("Unknown loader type: {loader_type_str}")),
        };
        LoaderConfig {
            enable: true,
            loader_type: Some(loader_type),
            build: args.loader_build.unwrap_or_else(|| "latest".into()),
            path: None,
            config: None,
        }
    } else {
        LoaderConfig {
            enable: false,
            ..Default::default()
        }
    };

    let options = LaunchOptions {
        path: args.game_dir.into(),
        version: args.version,
        authenticator: auth,
        timeout_secs: 30,
        download_concurrency: args.download_concurrency.unwrap_or(10),
        verify_concurrency: args.verify_concurrency.unwrap_or(4),
        memory: MemoryConfig {
            min: args.min_mem,
            max: args.max_mem,
        },
        java: JavaOptions {
            path: args.java_path.map(std::path::PathBuf::from),
            ..Default::default()
        },
        loader,
        screen: ScreenConfig::default(),
        verify: false,
        game_args: vec![],
        jvm_args: vec![],
        instance: args.instance,
        url: None,
        mcp: None,
        intel_enabled_mac: false,
        bypass_offline: true,
        skip_bundle_check: false,
        force_ipv4: false,
        dns: None,
    };

    let mut launcher = Launcher::new(options);
    let (tx, mut rx) = mpsc::channel::<LaunchEvent>(512);
    let app_clone = app.clone();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                LaunchEvent::Progress {
                    downloaded,
                    total,
                    kind,
                } => {
                    let _ = app_clone.emit(
                        "minecraft-progress",
                        MinecraftEvent::Progress {
                            kind,
                            downloaded,
                            total,
                        },
                    );
                }
                LaunchEvent::Data(line) => {
                    let _ = app_clone.emit("minecraft-output", MinecraftEvent::Data { line });
                }
                LaunchEvent::Close(code) => {
                    let _ = app_clone.emit("minecraft-exit", MinecraftEvent::Close { code });
                }
                LaunchEvent::Error(msg) => {
                    let _ =
                        app_clone.emit("minecraft-error", MinecraftEvent::Error { message: msg });
                }
                LaunchEvent::GameDownloadFinished => {
                    let _ = app_clone.emit("minecraft-ready", MinecraftEvent::GameDownloadFinished);
                }
                _ => {}
            }
        }
    });

    if args.download_only {
        launcher
            .download_game(tx)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        let mut child = launcher.start(tx).await.map_err(|e| e.to_string())?;
        let _ = app.emit("minecraft-ready", serde_json::json!({}));

        // Store the child PID for global process monitoring
        {
            let proc_state = app.state::<GameProcessState>();
            *proc_state.lock().await = child.id();
        }

        let (tx_stop, mut rx_stop) = tokio::sync::oneshot::channel::<()>();
        let signal = app.state::<GameStopSignal>();
        *signal.lock().await = Some(tx_stop);

        let app_clone2 = app.clone();

        // Poll the child process every 2 seconds instead of blocking on wait()
        // This ensures we detect process exit even if the library doesn't emit
        // LaunchEvent::Close (e.g. when game is killed via system close button)
        loop {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code().unwrap_or(-1);
                            let _ = app_clone2.emit("minecraft-exit", MinecraftEvent::Close { code });
                            break;
                        }
                        Ok(None) => {} // Still running, continue polling
                        Err(e) => {
                            let _ = app_clone2.emit("minecraft-error",
                                MinecraftEvent::Error { message: format!("游戏进程错误: {e}") });
                            break;
                        }
                    }
                }
                _ = &mut rx_stop => {
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    let _ = app.emit("minecraft-exit", MinecraftEvent::Close { code: 0 });
                    break;
                }
            }
        }

        // Clear the stop signal
        *signal.lock().await = None;
        // Clear the process state
        {
            let proc_state = app.state::<GameProcessState>();
            *proc_state.lock().await = None;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OobeSettings {
    pub locale: String,
    pub theme: String,
    pub font: String,
    pub java_path: String,
    pub account_type: String,
    pub account_name: String,
    pub oobe_completed: bool,
}

impl Default for OobeSettings {
    fn default() -> Self {
        Self {
            locale: "system".into(),
            theme: "system".into(),
            font: "__system_default__".into(),
            java_path: String::new(),
            account_type: "offline".into(),
            account_name: String::new(),
            oobe_completed: false,
        }
    }
}

fn get_minecraft_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                return dir.join(".minecraft");
            }
        }
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".minecraft")
}

fn get_settings_path() -> std::path::PathBuf {
    get_minecraft_dir().join("settings.json")
}

#[tauri::command]
pub fn init_oobe_environment() -> Result<OobeSettings, String> {
    let mc_dir = get_minecraft_dir();
    std::fs::create_dir_all(&mc_dir).map_err(|e| format!("Failed to create .minecraft: {}", e))?;

    let dirs = ["versions", "logs", "caches"];
    for d in &dirs {
        let path = mc_dir.join(d);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create .minecraft/{}: {}", d, e))?;
    }

    let settings = OobeSettings::default();
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(get_settings_path(), &json)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    Ok(settings)
}

#[tauri::command]
pub fn save_oobe_settings(settings: OobeSettings) -> Result<(), String> {
    let mc_dir = get_minecraft_dir();
    std::fs::create_dir_all(&mc_dir).map_err(|e| format!("Failed to create .minecraft: {}", e))?;
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(get_settings_path(), &json)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn rollback_oobe() -> Result<(), String> {
    let mc_dir = get_minecraft_dir();
    let settings_path = get_settings_path();
    if settings_path.exists() {
        std::fs::remove_file(&settings_path).ok();
    }
    if mc_dir.exists() {
        std::fs::remove_dir_all(&mc_dir)
            .map_err(|e| format!("Failed to remove .minecraft: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn check_oobe_completed() -> Result<bool, String> {
    let path = get_settings_path();
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;
    let settings: OobeSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;
    Ok(settings.oobe_completed)
}

#[tauri::command]
pub fn get_oobe_settings() -> Result<OobeSettings, String> {
    let path = get_settings_path();
    if !path.exists() {
        return Ok(OobeSettings::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;
    let settings: OobeSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;
    Ok(settings)
}

#[tauri::command]
pub fn get_minecraft_dir_string() -> String {
    get_minecraft_dir().to_string_lossy().to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentAccount {
    pub name: String,
    pub account_type: String,
    pub uuid: String,
}

#[tauri::command]
pub fn get_current_account() -> Result<CurrentAccount, String> {
    let path = get_settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;
        if let Ok(settings) = serde_json::from_str::<OobeSettings>(&content) {
            if !settings.account_name.is_empty() {
                let uuid = offline_uuid(&settings.account_name);
                return Ok(CurrentAccount {
                    name: settings.account_name,
                    account_type: settings.account_type,
                    uuid,
                });
            }
        }
    }
    let accounts = read_accounts();
    if let Some(first) = accounts.first() {
        return Ok(CurrentAccount {
            name: first.name.clone(),
            account_type: first.r#type.clone(),
            uuid: first.uuid.clone(),
        });
    }
    Ok(CurrentAccount {
        name: String::new(),
        account_type: String::new(),
        uuid: String::new(),
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoaderEntry {
    #[serde(rename = "type")]
    pub r#type: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceEntry {
    pub name: String,
    pub version: String,
    pub version_type: String,
    pub loader: Option<LoaderEntry>,
    pub icon: Option<String>,
    pub installed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionDownloads {
    pub client: VersionDownload,
    pub server: Option<VersionDownload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionJson {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub downloads: VersionDownloads,
    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<VersionDownload>,
}

#[tauri::command]
pub fn finish_oobe(settings: OobeSettings, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut s = settings;
    s.oobe_completed = true;
    let json = serde_json::to_string_pretty(&s)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(get_settings_path(), &json)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }

    if let Some(oobe_window) = app_handle.get_webview_window("oobe") {
        let _ = oobe_window.close();
    }

    let _ = app_handle.emit("account-refresh", ());

    Ok(())
}

fn get_instances_list_path() -> std::path::PathBuf {
    get_minecraft_dir().join("in_versions_list.json")
}

fn read_instances_list() -> Vec<InstanceEntry> {
    let path = get_instances_list_path();
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_instances_list(instances: &[InstanceEntry]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(instances)
        .map_err(|e| format!("Failed to serialize instances list: {}", e))?;
    std::fs::write(get_instances_list_path(), &json)
        .map_err(|e| format!("Failed to write instances list: {}", e))
}

#[tauri::command]
pub async fn install_instance(
    app: AppHandle,
    name: String,
    mc_version: String,
    version_type: String,
    loader_type: Option<String>,
    loader_version: Option<String>,
) -> Result<(), String> {
    let mc_dir = get_minecraft_dir();
    let instance_dir = mc_dir.join("versions").join(&name);
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("Failed to create instance dir: {}", e))?;

    let _ = app.emit(
        "install-progress",
        serde_json::json!({
            "step": "manifest",
            "progress": 0.0
        }),
    );

    let manifest_url = "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json";
    let manifest_resp = reqwest::get(manifest_url)
        .await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    let manifest: VersionManifest = manifest_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse version manifest: {}", e))?;

    let version_info = manifest
        .versions
        .iter()
        .find(|v| v.id == mc_version)
        .ok_or_else(|| format!("Version {} not found in manifest", mc_version))?;

    let _ = app.emit(
        "install-progress",
        serde_json::json!({
            "step": "version_json",
            "progress": 0.2
        }),
    );

    let version_json_resp = reqwest::get(&version_info.url)
        .await
        .map_err(|e| format!("Failed to fetch version JSON: {}", e))?;
    let version_json: VersionJson = version_json_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

    let version_json_path = instance_dir.join(format!("{}.json", &name));
    let version_json_content = serde_json::to_string_pretty(&version_json)
        .map_err(|e| format!("Failed to serialize version JSON: {}", e))?;
    std::fs::write(&version_json_path, &version_json_content)
        .map_err(|e| format!("Failed to write version JSON: {}", e))?;

    let _ = app.emit(
        "install-progress",
        serde_json::json!({
            "step": "client_jar",
            "progress": 0.4
        }),
    );

    let jar_url = &version_json.downloads.client.url;
    let jar_path = instance_dir.join(format!("{}.jar", &name));

    let jar_resp = reqwest::get(jar_url)
        .await
        .map_err(|e| format!("Failed to fetch client jar: {}", e))?;

    let total_size = jar_resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(&jar_path)
        .map_err(|e| format!("Failed to create jar file: {}", e))?;
    let mut stream = jar_resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write jar data: {}", e))?;

        if total_size > 0 {
            let progress = 0.4 + (downloaded as f64 / total_size as f64) * 0.4;
            let _ = app.emit(
                "install-progress",
                serde_json::json!({
                    "step": "client_jar",
                    "progress": progress
                }),
            );
        }
    }

    let _ = app.emit(
        "install-progress",
        serde_json::json!({
            "step": "finalize",
            "progress": 0.9
        }),
    );

    let loader = loader_type.map(|lt| LoaderEntry {
        r#type: lt,
        version: loader_version.unwrap_or_default(),
    });

    let new_entry = InstanceEntry {
        name: name.clone(),
        version: mc_version,
        version_type,
        loader,
        icon: None,
        installed: false,
    };

    let mut instances = read_instances_list();
    instances.push(new_entry);
    write_instances_list(&instances)?;

    let _ = app.emit(
        "install-progress",
        serde_json::json!({
            "step": "done",
            "progress": 1.0
        }),
    );

    Ok(())
}

#[tauri::command]
pub fn get_instances_list() -> Result<Vec<InstanceEntry>, String> {
    Ok(read_instances_list())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountEntry {
    pub name: String,
    pub r#type: String,
    pub uuid: String,
}

fn get_accounts_path() -> std::path::PathBuf {
    get_minecraft_dir().join("accounts.json")
}

fn read_accounts() -> Vec<AccountEntry> {
    let path = get_accounts_path();
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_accounts(accounts: &[AccountEntry]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(accounts)
        .map_err(|e| format!("Failed to serialize accounts: {}", e))?;
    std::fs::write(get_accounts_path(), &json)
        .map_err(|e| format!("Failed to write accounts: {}", e))
}

#[tauri::command]
pub fn get_accounts() -> Result<Vec<AccountEntry>, String> {
    let mut accounts = read_accounts();
    if accounts.is_empty() {
        let settings_path = get_settings_path();
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(settings) = serde_json::from_str::<OobeSettings>(&content) {
                    if !settings.account_name.is_empty() {
                        let uuid = offline_uuid(&settings.account_name);
                        accounts.push(AccountEntry {
                            name: settings.account_name.clone(),
                            r#type: settings.account_type.clone(),
                            uuid,
                        });
                        write_accounts(&accounts).ok();
                    }
                }
            }
        }
    }
    Ok(accounts)
}

#[tauri::command]
pub fn add_account(name: String, account_type: String) -> Result<Vec<AccountEntry>, String> {
    let uuid = if name.is_empty() {
        String::new()
    } else {
        offline_uuid(&name)
    };
    let entry = AccountEntry {
        name,
        r#type: account_type,
        uuid,
    };
    let mut accounts = read_accounts();
    accounts.push(entry);
    write_accounts(&accounts)?;
    Ok(accounts)
}

#[tauri::command]
pub fn remove_account(name: String) -> Result<Vec<AccountEntry>, String> {
    let mut accounts = read_accounts();
    accounts.retain(|a| a.name != name);
    write_accounts(&accounts)?;
    Ok(accounts)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSettings {
    pub icon: Option<String>,
    pub skip_launcher: bool,
    pub auto_connect_address: Option<String>,
    pub java_version: Option<String>,
    pub auto_memory: bool,
    pub min_memory: String,
    pub max_memory: String,
    pub jvm_args: String,
    pub game_args: String,
    pub download_concurrency: u32,
    pub verify_concurrency: u32,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            icon: None,
            skip_launcher: false,
            auto_connect_address: None,
            java_version: None,
            auto_memory: true,
            min_memory: "1024M".into(),
            max_memory: "2048M".into(),
            jvm_args: String::new(),
            game_args: String::new(),
            download_concurrency: 10,
            verify_concurrency: 4,
        }
    }
}

fn get_instance_settings_path(name: &str) -> std::path::PathBuf {
    get_minecraft_dir().join("versions").join(name).join("settings.json")
}

#[tauri::command]
pub fn get_instance_settings(instance_name: String) -> Result<InstanceSettings, String> {
    let path = get_instance_settings_path(&instance_name);
    if !path.exists() {
        return Ok(InstanceSettings::default());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("无法读取实例设置: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("无法解析实例设置: {e}"))
}

#[tauri::command]
pub fn save_instance_settings(
    instance_name: String,
    settings: InstanceSettings,
) -> Result<(), String> {
    let path = get_instance_settings_path(&instance_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("无法创建设置目录: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("无法序列化设置: {e}"))?;
    std::fs::write(&path, &json).map_err(|e| format!("无法保存设置: {e}"))
}

#[tauri::command]
pub async fn stop_game(app: AppHandle) -> Result<(), String> {
    let signal = app.state::<GameStopSignal>();
    let mut guard = signal.lock().await;
    if let Some(tx) = guard.take() {
        let _ = tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn check_game_running(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<GameProcessState>();
    let pid = {
        let guard = state.lock().await;
        *guard
    };

    match pid {
        Some(pid) => {
            let running = tokio::task::spawn_blocking(move || {
                let mut system = sysinfo::System::new();
                system.refresh_processes(
                    sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]),
                );
                system.process(sysinfo::Pid::from_u32(pid)).is_some()
            })
            .await
            .map_err(|e| format!("Failed to check process: {e}"))?;
            Ok(running)
        }
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn open_crash_shell(app: AppHandle, report: CrashReport) -> Result<(), String> {
    let state = app.state::<CrashReportState>();
    {
        let mut guard = state.lock().await;
        *guard = Some(report);
    }

    // check if window already exists, reuse it
    if let Some(window) = app.get_webview_window("crash-shell") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    use tauri::WebviewWindowBuilder;
    let crash_builder = WebviewWindowBuilder::new(
        &app,
        "crash-shell",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("崩溃报告")
    .inner_size(750.0, 600.0)
    .resizable(true)
    .center();

    #[cfg(not(target_os = "macos"))]
    let crash_builder = crash_builder.decorations(false);

    let crash_window = crash_builder.build().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    crash_window
        .set_title_bar_style(tauri::TitleBarStyle::Transparent)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_crash_report(app: AppHandle) -> Result<Option<CrashReport>, String> {
    let state = app.state::<CrashReportState>();
    let guard = state.lock().await;
    Ok(guard.clone())
}

#[tauri::command]
pub async fn clear_crash_report(app: AppHandle) -> Result<(), String> {
    let state = app.state::<CrashReportState>();
    let mut guard = state.lock().await;
    *guard = None;
    Ok(())
}

#[tauri::command]
pub async fn open_log_folder(app: AppHandle) -> Result<(), String> {
    let mc_dir = get_minecraft_dir();
    let logs_dir = mc_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("无法创建日志目录: {e}"))?;
    let opener = app.state::<tauri_plugin_opener::Opener<tauri::Wry>>();
    opener
        .open_path(logs_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("无法打开日志文件夹: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn export_crash_log(app: AppHandle, content: String) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<std::path::PathBuf>>();
    app.dialog()
        .file()
        .add_filter("日志文件", &["log", "txt"])
        .set_file_name("crash-report.log")
        .save_file(move |path| {
            let _ = tx.send(path.and_then(|p| p.into_path().ok()));
        });
    if let Some(path) = rx.await.unwrap_or(None) {
        std::fs::write(&path, &content)
            .map_err(|e| format!("无法保存日志文件: {e}"))?;
    }
    Ok(())
}
