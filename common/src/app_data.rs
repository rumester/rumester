use std::{fs, path::PathBuf};

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

pub fn kill_prefix(app_name: &String) -> Result<(), String> {
    get_wine(app_name).kill()
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

pub fn get_webview_state_dir(wine: &Option<Wine>) -> PathBuf {
    if let Some(wine) = wine {
        return wine.prefix_path.join("webview_state");
    }
    get_appdata_dir().join("webview_state")
}

pub fn get_webview_installed(wine: &Option<Wine>) -> bool {
    read_state_file(get_webview_state_dir(wine))
}

pub fn set_webview_installed(wine: &Option<Wine>, installed: bool) {
    write_state_file(get_webview_state_dir(wine), installed);
}

pub fn get_local_appdata_dir(app_name: &String) -> PathBuf {    
    #[cfg(target_os = "windows")] {
        let local_data_dir = dirs::data_local_dir().unwrap();
        let log_dir = local_data_dir.join("Roblox/Logs");
        return log_dir;
    }

    let username = whoami::realname();
    let wine_prefix = ensure_prefix_exists(app_name);
    let log_dir = wine_prefix
        .join("drive_c/users/")
        .join(username)
        .join("AppData/Local");
    log_dir
}
