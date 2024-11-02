use std::{process, fs, path::PathBuf};

use winers::Wine;

use crate::mirror::Package;

pub fn get_appdata_dir() -> PathBuf {
    let appdata = dirs::data_local_dir().unwrap().join("rumester");
    if !appdata.exists() {
        fs::create_dir_all(&appdata).unwrap();
    }
    appdata
}

pub fn get_cache_dir() -> PathBuf {
    get_appdata_dir().join("cache")
}

pub fn get_deployments_download_dir() -> PathBuf {
    get_appdata_dir().join("downloads")
}

pub fn get_deployments_dir() -> PathBuf {
    get_appdata_dir().join("deployments")
}

pub fn get_prefix_dir() -> PathBuf {
    get_appdata_dir().join("prefixes")
}

pub fn ensure_prefix_exists(app_name: &String) -> PathBuf {
    let prefix_path = get_prefix_dir().join(app_name);
    if !prefix_path.exists() {
        fs::create_dir_all(&prefix_path).unwrap();
    }
    prefix_path
}

pub fn get_wineroot() -> Option<PathBuf> {
    if let Ok(wineroot) = std::env::var("WINEROOT") {
        Some(PathBuf::from(wineroot))
    } else {
        None
    }
}

pub fn get_wineroot_string() -> Option<String> {
    if let Some(wineroot) = get_wineroot() {
        Some(wineroot.to_str().unwrap().to_string())
    } else {
        None
    }
}

pub fn get_wine(app_name: &String) -> Wine {
    winers::Wine::new(
        ensure_prefix_exists(app_name).to_str().unwrap(),
        get_wineroot_string(),
    )
}

pub fn get_binary_type(app: &str) -> &'static str {
    match app {
        "player" => "WindowsPlayer",
        "studio" => "WindowsStudio64",
        _ => panic!("Invalid binary type."),
    }
}

pub fn get_binary_name(app: &str) -> &'static str {
    match app {
        "player" => "RobloxPlayerBeta.exe",
        "studio" => "RobloxStudioBeta.exe",
        _ => panic!("Invalid binary type.")
    }
}

pub fn query_reg_key(app_name: &String, key: &str, value: &str) -> Result<Option<String>, String> {
    #[cfg(target_os = "linux")] {
        return get_wine(app_name).reg_query(key, value);
    }
    #[cfg(target_os = "windows")] {
        let output = process::Command::new("reg")
            .args(["query", key, "/v", value])
            .output()
            .map_err(|e| format!("Failed to execute reg: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to query registry key: {}", stderr));
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                if line.contains(value) {
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if tokens.len() >= 3 {
                        return Ok(Some(tokens[2].to_string()));
                    } else {
                        return Err("Unexpected format in output.".to_string());
                    }
                }
            }
        }

        Ok(None)
    }
}

pub fn kill_prefix(app_name: &String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        get_wine(app_name).kill()
    }
    #[cfg(target_os = "windows")]
    {
        let binary_name = get_binary_name(&app_name);
        let output = process::Command::new("taskkill")
            .args(["/IM", &binary_name, "/F"])
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Killing {} failed: {}", &app_name, stderr));
        }

        Ok(())
    }
}

pub fn cleanup_app(app_name: &String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = kill_prefix(app_name) {
            return Err(format!("Failed to kill prefix: {e}"));
        }
    }

    Ok(())
}

pub fn get_download_dir() -> PathBuf {
    let path = get_cache_dir().join("downloads");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create download cache dir!");
    }
    path
}

pub fn get_package_dir(package: &Package) -> PathBuf {
    let packages_dir = get_download_dir().join("packages");
    let path = packages_dir.join(package.checksum.to_string());
    if !packages_dir.exists() {
        fs::create_dir_all(&packages_dir).expect("Failed to create download cache dir!");
    }
    path
}

pub fn read_state_file(path: PathBuf) -> bool {
    if let Ok(state) = fs::read_to_string(path) {
        state == "1"
    } else {
        false
    }
}

pub fn write_state_file(path: PathBuf, installed: bool) {
    fs::write(path, if installed { "1" } else { "0" })
        .expect("Failed to set DXVK installed state.");
}

fn get_dxvk_state_dir(wine: &Wine) -> PathBuf {
    wine.prefix_path.join("dxvk_state")
}

pub fn get_dxvk_installed(wine: &Wine) -> bool {
    read_state_file(get_dxvk_state_dir(wine))
}

pub fn set_dxvk_installed(wine: &Wine, installed: bool) {
    write_state_file(get_dxvk_state_dir(wine), installed)
}


fn webview_version_check(version: &str) -> bool {
    let minimum_parts: Vec<i32> = "109.0.1518.140"
        .split('.')
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    let version_parts: Vec<i32> = version
        .split('.')
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    if minimum_parts[0] < version_parts[0] {
        return true;
    } else {
        for (minv, userv) in minimum_parts.iter().zip(version_parts.iter()) {
            if (minv <= userv) {
                continue;
            } else {
                return false;
            }
        }
        return true;
    }
}

pub fn get_webview_installed(app_name: &String) -> bool {
    let webview_version = query_reg_key(app_name, r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "pv");

    match webview_version {
        Ok(Some(version)) => {
            return webview_version_check(&version)
        },
        Ok(None) => {
            return false;
        },
        Err(_) => {
            return false;
        }
    };
}

pub fn get_local_appdata_dir(app_name: &String) -> PathBuf {    
    #[cfg(target_os = "windows")] {
        return dirs::data_local_dir().unwrap();
    }

    let username = whoami::realname();
    let wine_prefix = ensure_prefix_exists(app_name);
    let log_dir = wine_prefix
        .join("drive_c/users/")
        .join(username)
        .join("AppData/Local");
    log_dir
}
