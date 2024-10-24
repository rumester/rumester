use common::client_settings::get_client_version;

#[tokio::main]
async fn main() {
    let latest_version = get_client_version("WindowsStudio64", None).await;
    if let Some(latest_version) = latest_version.ok() {
        println!("Latest version: {} (GUID {})", latest_version.version, latest_version.client_version_upload);
    } else {
        println!("Failed to fetch version.");
    }
}
