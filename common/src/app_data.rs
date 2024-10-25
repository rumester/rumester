use std::{fs, path::PathBuf};

pub fn get_appdata_dir() -> PathBuf {
    dirs::data_local_dir().unwrap().join("rumester")
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

pub fn get_download_dir() -> PathBuf {
    let path = get_cache_dir().join("downloads");
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create download cache dir!");
    }
    path
}
