use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub fn get_last_modified_file(dir_path: &str) -> io::Result<Option<String>> {
    let path = Path::new(dir_path);
    
    fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            if metadata.is_file() {
                Some((entry.file_name().to_string_lossy().into_owned(), metadata.modified().ok()?))
            } else {
                None
            }
        })
        .max_by_key(|&(_, modified_time)| modified_time)
        .map(|(file_name, _)| file_name)
        .map_or(Ok(None), |name| Ok(Some(name)))
}

#[cfg(target_os = "linux")]
pub fn get_log_dir(prefix: PathBuf) -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    {
        let user = std::env::var("USER").map_err(|e| 
            format!("Failed to obtain USER environment variable: {}", e)
        )?;

        Ok(prefix.join(format!("drive_c/users/{}/AppData/Local/Roblox/logs", user)))
    }
}

#[cfg(target_os = "windows")]
pub fn get_log_dir() -> Result<PathBuf, String> {
    let localappdata = std::env::var("LocalAppData").map_err(|e| 
        format!("Failed to obtain LocalAppData environment variable: {}", e)
    )?;

    Ok(format!("{}/Roblox/logs", localappdata))
}