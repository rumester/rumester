use common::{
    client_settings::get_client_version,
    mirror::{get_mirror, get_mirror_packages},
};

#[tokio::main]
async fn main() {
    let latest_version = get_client_version("WindowsStudio64", None).await;
    if let Some(latest_version) = latest_version.ok() {
        let mirror = get_mirror().await.expect("Failed to find a valid mirror.");
        println!(
            "Fetching version {} (GUID {}) from {}",
            latest_version.version, latest_version.client_version_upload, mirror
        );
        let packages = get_mirror_packages(mirror.as_str(), latest_version)
            .await
            .expect(format!("Failed to fetch packages from {mirror}").as_str());
        for package in packages {
            println!(
                "Package {}: checksum: {}, size: {}, zipsize: {}",
                package.name, package.checksum, package.size, package.zipsize
            );
        }
    } else {
        println!("Failed to fetch version.");
    }
}
