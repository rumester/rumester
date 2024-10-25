use std::{fs, io, path::PathBuf};

use serde::Deserialize;

use crate::app_data::get_deployments_dir;

#[derive(Deserialize)]
pub struct ClientDeployment {
    pub version: String,
    #[serde(rename = "clientVersionUpload")]
    pub client_version_upload: String,
}

impl ClientDeployment {
    pub fn get_install_dir(&self) -> PathBuf {
        get_deployments_dir().join(&self.client_version_upload)
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
