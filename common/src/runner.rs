use std::{env, path::PathBuf, process::Child};

use winers::{get_latest_dxvk, install_dxvk, Wine};

use crate::{
    app_data::{
        get_dxvk_installed, get_webview_installed, get_wine, set_dxvk_installed,
        set_webview_installed,
    },
    client_settings::ClientDeployment,
};

pub async fn run_windows_binary(binary_file: PathBuf, app_name: &String) -> Result<Child, String> {
    println!("Running {}", binary_file.to_str().unwrap());
    #[cfg(target_os = "windows")]
    {
        let child = std::process::Command::new(binary_file).spawn().unwrap();
        return Ok(child);
    }
    let wine = get_wine(&app_name);
    if let Err(e) = wine.init() {
        return Err(format!("Error initializing wine: {}", e));
    }
    if app_name == "studio" {
        // for some reason Roblox Studio likes to explode when on 96 DPi why I don't know but hey it fixes it!
        if let Err(e) = wine.reg_add(
            r"HKEY_CURRENT_USER\Control Panel\Desktop",
            "LogPixels",
            "REG_DWORD",
            "97",
        ) {
            eprintln!("Failed to set DPI for Studio prefix, this may result in Studio crashing after the splash screen closes. Error info: {e}");
        };
    }

    if !get_dxvk_installed(&wine) {
        match get_latest_dxvk().await {
            Ok(latest_dxvk) => {
                if let Err(e) = install_dxvk(&wine, latest_dxvk.as_str()).await {
                    eprintln!("Error installing dxvk: {}", e.to_string());
                } else {
                    set_dxvk_installed(&wine, true);
                }
            }
            Err(e) => eprintln!("Error fetching latest dxvk: {e}"),
        }
    }

    let winedebug = env::var("WINEDEBUG").unwrap_or_else(|_| "-all".to_string());

    let mut cmd = wine.cmd();
    cmd.arg(binary_file);
    cmd.env("WINEDEBUG", winedebug);
    let child = cmd.spawn().unwrap();
    Ok(child)
}

#[cfg(target_os = "linux")]
pub fn set_wine_windows_version(wine: &Wine, version: &str) {
    let mut cmd = wine.cmd();
    cmd.arg("winecfg").arg("/v").arg(version);
    cmd.output().expect("Failed to run winecfg");
}

pub async fn install_webview2(
    app_name: &String,
    deployment: &ClientDeployment,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let wine = Some(get_wine(app_name));
    #[cfg(target_os = "windows")]
    let wine = None;
    if !get_webview_installed(&wine) {
        #[cfg(target_os = "linux")]
        set_wine_windows_version(wine.as_ref().unwrap(), "win7");

        match run_windows_binary(deployment.get_webview_installer_dir(), app_name).await {
            Ok(child) => {
                set_webview_installed(&wine, true);
                let output = child.wait_with_output();
                #[cfg(target_os = "linux")]
                {
                    let wine = wine.unwrap();
                    if let Err(e) = wine.kill() {
                        eprintln!("Failed to kill wine: {e}");
                    }
                }
                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            return Err(format!(
                                "Webview installer exited with non-zero exit status: {}",
                                output.status
                            ));
                        }
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to install webview: {}", e.to_string()).into()),
                }
            }
            Err(e) => Err(format!("Failed to run webview installer: {e}").into()),
        }
    } else {
        Ok(())
    }
}
