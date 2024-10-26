use std::{fs, io, path::PathBuf};

use serde::Deserialize;

use crate::app_data::get_deployments_dir;

#[derive(Deserialize, Clone)]
pub struct ClientDeployment {
    pub version: String,
    #[serde(rename = "clientVersionUpload")]
    pub client_version_upload: String,
}

impl ClientDeployment {
    pub fn get_install_dir(&self) -> PathBuf {
        get_deployments_dir().join(&self.client_version_upload)
    }

    pub fn get_webview_installer_dir(&self) -> PathBuf {
        self.get_install_dir()
            .join("WebView2RuntimeInstaller/MicrosoftEdgeWebview2Setup.exe")
    }

    fn get_webview_state_dir(&self) -> PathBuf {
        let path = self.get_install_dir().join("rumester_webview_state");
        if !path.exists() {
            fs::write(&path, "0").expect("Failed to create webview state file");
        }
        path
    }

    pub fn get_webview_installed(&self) -> bool {
        let state_file = fs::read_to_string(self.get_webview_state_dir())
            .expect("Failed to read webview state file!");

        state_file == "1"
    }

    pub fn set_webview_installed(&self, installed: bool) {
        fs::write(
            self.get_webview_state_dir(),
            if installed { "1" } else { "0" },
        )
        .expect("Failed to write webview state!");
    }

    pub fn is_installed(&self) -> bool {
        self.get_install_dir().exists()
    }

    pub fn setup_deployment_dir(&self) -> Result<(), io::Error> {
        let target_dir = self.get_install_dir();
        if target_dir.exists() {
            Ok(())
        } else {
            fs::create_dir_all(&target_dir)
        }
    }
}

pub async fn get_client_version(
    binary_type: &str,
    channel: Option<&str>,
) -> Result<ClientDeployment, Box<dyn std::error::Error>> {
    let mut url = format!(
        "https://clientsettings.roblox.com/v2/client-version/{}",
        binary_type
    );
    if let Some(channel) = channel {
        url.push_str(format!("/channel/{}", channel).as_str());
    }
    let res = reqwest::get(format!(
        "https://clientsettings.roblox.com/v2/client-version/{}",
        binary_type
    ))
    .await?
    .json::<ClientDeployment>()
    .await?;

    Ok(res)
}
