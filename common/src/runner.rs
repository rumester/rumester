use std::{
    path::PathBuf,
    process::{Child, Stdio},
};

use crate::{
    app_data::{ensure_prefix_exists, get_wineroot_string},
    client_settings::ClientDeployment,
};

pub fn run_windows_binary(binary_file: PathBuf, app_name: &String) -> Result<Child, String> {
    println!("Running {}", binary_file.to_str().unwrap());
    #[cfg(target_os = "windows")]
    {
        let child = std::process::Command::new(binary_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        return Ok(child);
    }
    let prefix_path = ensure_prefix_exists(&app_name);
    let wine = winers::Wine::new(prefix_path.to_str().unwrap(), get_wineroot_string());
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

pub async fn install_webview2(
    app_name: &String,
    deployment: &ClientDeployment,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let wine = winers::Wine::new(
            ensure_prefix_exists(app_name).to_str().unwrap(),
            get_wineroot_string(),
        );
        let mut cmd = wine.cmd();
        cmd.arg("winecfg").arg("/v").arg("win7");
        cmd.spawn().unwrap().wait().unwrap();
    }
    if !deployment.get_webview_installed() {
        if let Ok(mut child) = run_windows_binary(deployment.get_webview_installer_dir(), app_name)
        {
            deployment.set_webview_installed(true);
            if child.wait().is_ok() {
                Ok(())
            } else {
                Err("Failed to install webview!".into())
            }
        } else {
            Err("Failed to run webview installer!".into())
        }
    } else {
        Ok(())
    }
}
