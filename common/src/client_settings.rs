use serde::Deserialize;

#[derive(Deserialize)]
pub struct ClientVersionResponse {
    pub version: String,
    #[serde(rename = "clientVersionUpload")]
    pub client_version_upload: String,
}

pub async fn get_client_version(
    binary_type: &str,
    channel: Option<&str>,
) -> Result<ClientVersionResponse, Box<dyn std::error::Error>> {
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
    .json::<ClientVersionResponse>()
    .await?;

    Ok(res)
}
