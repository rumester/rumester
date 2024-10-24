use common::{
    client_settings::get_client_version,
    download::{download_package, install_package, write_app_settings_xml},
    mirror::{get_mirror, get_mirror_packages},
};

#[tokio::main]
async fn main() {
    let latest_version = get_client_version("WindowsStudio64", None).await;
    if let Some(latest_version) = &latest_version.ok() {
        match latest_version.setup_deployment_dir() {
            Ok(_) => {}
            Err(e) => panic!("Failed to setup deployment dir! {e}"),
        }
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
            let data = download_package(mirror.as_str(), latest_version, &package)
                .await
                .expect(format!("Failed to download {}", package.name).as_str());
            install_package(&package, &latest_version, &data).expect("Failed to install package.");
        }
        write_app_settings_xml(&latest_version);
    } else {
        println!("Failed to fetch version.");
    }
}
