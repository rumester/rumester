use std::{
    fs,
    path::PathBuf,
    process::{Child, Stdio},
};

use crate::app_data::{get_prefix_dir, get_wineroot};

pub fn run_windows_binary(binary_file: PathBuf, app_name: String) -> Result<Child, String> {
    println!("Running {}", binary_file.to_str().unwrap());
    let prefix_path = get_prefix_dir().join(app_name);
    if !prefix_path.exists() {
        fs::create_dir_all(&prefix_path).unwrap();
    }
    let wineroot = if let Some(wineroot) = get_wineroot() {
        Some(wineroot.to_str().unwrap().to_string())
    } else {
        None
    };
    let wine = winers::Wine::new(
        prefix_path.to_str().unwrap(),
        wineroot,
    );
    if let Err(e) = wine.init() {
        panic!("Error initializing wine: {e}");
    }
    let mut cmd = wine.cmd();
    cmd.arg(binary_file);
    let child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    Ok(child)
}
