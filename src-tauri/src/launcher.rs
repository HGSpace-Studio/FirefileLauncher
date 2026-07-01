use minecraft_java_rs_core::launcher::events::LaunchEvent;
use minecraft_java_rs_core::launcher::options::{
    JavaOptions, LaunchOptions, LoaderConfig, MemoryConfig, ScreenConfig,
};
use minecraft_java_rs_core::launcher::Launcher;
use minecraft_java_rs_core::models::loader::LoaderType;
use minecraft_java_rs_core::models::minecraft::Authenticator;
use minecraft_java_rs_core::utils::auth::offline_uuid;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

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
pub struct SystemMemory {
    pub total_mb: u64,
    pub used_mb: u64,
}

#[tauri::command]
pub async fn get_system_memory() -> Result<SystemMemory, String> {
    let mem = tokio::task::spawn_blocking(|| {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        (sys.total_memory() / 1024 / 1024, sys.used_memory() / 1024 / 1024)
    }).await.map_err(|e| format!("{e}"))?;
    Ok(SystemMemory { total_mb: mem.0, used_mb: mem.1 })
}

#[tauri::command]
pub async fn get_java_versions() -> Result<Vec<JavaInstall>, String> {
    let results = tokio::task::spawn_blocking(|| discover_java()).await
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
                let java_bin = entry.path().join("Contents").join("Home").join("bin").join("java");
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
    let java_cmd = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
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
    let version = stderr
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
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
    pub loader_type: Option<String>,
    pub loader_build: Option<String>,
    pub instance: Option<String>,
    pub download_only: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MinecraftEvent {
    Progress { kind: String, downloaded: u64, total: u64 },
    Data { line: String },
    Close { code: i32 },
    Error { message: String },
    GameDownloadFinished,
}

#[tauri::command]
pub async fn launch_minecraft(
    app: AppHandle,
    args: LaunchArgs,
) -> Result<(), String> {
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
        download_concurrency: 10,
        verify_concurrency: 4,
        memory: MemoryConfig {
            min: args.min_mem,
            max: args.max_mem,
        },
        java: JavaOptions::default(),
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
                    let _ = app_clone.emit(
                        "minecraft-output",
                        MinecraftEvent::Data { line },
                    );
                }
                LaunchEvent::Close(code) => {
                    let _ = app_clone.emit(
                        "minecraft-exit",
                        MinecraftEvent::Close { code },
                    );
                }
                LaunchEvent::Error(msg) => {
                    let _ = app_clone.emit(
                        "minecraft-error",
                        MinecraftEvent::Error { message: msg },
                    );
                }
                LaunchEvent::GameDownloadFinished => {
                    let _ = app_clone.emit(
                        "minecraft-ready",
                        MinecraftEvent::GameDownloadFinished,
                    );
                }
                _ => {}
            }
        }
    });

    if args.download_only {
        launcher.download_game(tx).await.map_err(|e| e.to_string())?;
    } else {
        let mut child = launcher.start(tx).await.map_err(|e| e.to_string())?;
        child.wait().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}
