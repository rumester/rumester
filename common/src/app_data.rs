use std::path::PathBuf;

pub fn get_appdata_dir() -> PathBuf {
    dirs::data_local_dir().unwrap().join("rumester")
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

pub fn get_wineroot() -> Option<PathBuf> {
    if let Ok(wineroot) = std::env::var("WINEROOT") {
        Some(PathBuf::from(wineroot))
    } else {
        None
    }
}
