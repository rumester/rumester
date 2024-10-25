use std::io::Read;

use clap::{arg, command};
use common::{
    client_settings::get_client_version,
    download::{download_package, install_package, write_app_settings_xml},
    mirror::{get_mirror, get_mirror_packages},
    runner::{install_webview2, run_windows_binary},
};

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(arg!([app] "Roblox app to operate on").value_parser(["player", "studio"]))
        .arg(arg!([operation] "Operation to run").value_parser(["run"]))
        .get_matches();

    if let Some(app) = matches.get_one::<String>("app") {
        let binary_type = match app.as_str() {
            "player" => "WindowsPlayer",
            "studio" => "WindowsStudio64",
            _ => panic!("Invalid binary type."),
        };
        let binary_name = match app.as_str() {
            "player" => "RobloxPlayerBeta.exe",
            "studio" => "RobloxStudioBeta.exe",
            _ => panic!("Invalid binary type."),
        };
        let latest_version = get_client_version(binary_type, None).await;
        if let Some(latest_version) = &latest_version.ok() {
            if !latest_version.is_installed() {
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
                    if package.name == "RobloxPlayerLauncher.exe" {
                        continue;
                    }
                    let data = download_package(mirror.as_str(), latest_version, &package)
                        .await
                        .expect(format!("Failed to download {}", package.name).as_str());
                    install_package(&package, &latest_version, &data)
                        .expect("Failed to install package.");
                }
                write_app_settings_xml(&latest_version);
            }
            if install_webview2(app.into(), &latest_version).await.is_err() {
                eprintln!("Failed to install Webview2!");
            } else {
                println!("Installed Webview2.");
            }
            let child = run_windows_binary(
                latest_version.get_install_dir().join(binary_name),
                app.into(),
            );
            if let Ok(mut child) = child {
                let stdout = child.stdout.as_mut().unwrap();
                let stderr = child.stderr.as_mut().unwrap();

                loop {
                    let mut buf = [0; 1024];
                    let read_bytes = stdout.read(&mut buf).unwrap();
                    if read_bytes == 0 {
                        break;
                    }
                    print!("{}", String::from_utf8_lossy(&buf[..read_bytes]));

                    let mut err_buf = [0; 1024];
                    let read_bytes = stderr.read(&mut err_buf).unwrap();
                    if read_bytes == 0 {
                        break;
                    }
                    eprint!("{}", String::from_utf8_lossy(&err_buf[..read_bytes]));
                }

                let status = child.wait().unwrap();
                println!("binary exited with status: {}", status);
            }
        } else {
            println!("Failed to fetch version.");
        }
    }
}
