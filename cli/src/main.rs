use clap::{arg, command};
use common::{
    app_data::{get_wine, kill_prefix},
    client_settings::get_client_version,
    download::{format_file_size, download_package, install_package, write_app_settings_xml},
    mirror::{get_mirror, get_mirror_packages},
    runner::{install_webview2, run_windows_binary},
};

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(arg!([app] "Roblox app to operate on").value_parser(["player", "studio"]))
        .arg(arg!([operation] "Operation to run").value_parser(["run", "kill", "winecfg"]))
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

        if let Some(operation) = matches.get_one::<String>("operation") {
            match operation.as_str() {
                "run" => {
                    let latest_version = get_client_version(binary_type, None).await;
                    if let Some(latest_version) = &latest_version.ok() {
                        if !latest_version.is_installed() {
                            match latest_version.setup_deployment_dir() {
                                Ok(_) => {}
                                Err(e) => panic!("Failed to setup deployment dir! {e}"),
                            }
                            let mirror =
                                get_mirror().await.expect("Failed to find a valid mirror.");
                            println!(
                                "Fetching version {} (GUID {}) from {}",
                                latest_version.version,
                                latest_version.client_version_upload,
                                mirror
                            );
                            let packages = get_mirror_packages(mirror.as_str(), latest_version)
                                .await
                                .expect(format!("Failed to fetch packages from {mirror}").as_str());
                            let tasks: Vec<_> = packages
                                .clone()
                                .iter()
                                .map(|package| {
                                    let package: common::mirror::Package = package.clone();
                                    let mirror = mirror.clone();
                                    let latest_version = latest_version.clone();
                                    tokio::spawn(async move {
                                        println!(
                                            "Package {}: checksum: {}, size: {}, zipsize: {}",
                                            package.name,
                                            package.checksum,
                                            format_file_size(package.size),
                                            format_file_size(package.zipsize)
                                        );
                                        if package.name == "RobloxPlayerLauncher.exe" {
                                            return;
                                        }
                                        let data = download_package(
                                            mirror.as_str(),
                                            &latest_version,
                                            &package,
                                        )
                                        .await
                                        .expect(
                                            format!("Failed to download {}", package.name).as_str(),
                                        );
                                        install_package(&package, &latest_version, &data)
                                            .expect("Failed to install package.");
                                    })
                                })
                                .collect();
                            for task in tasks {
                                task.await.expect("Failed to run task");
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
                        )
                        .await;
                        if let Ok(child) = child {
                            let output = child.wait_with_output().expect("Failed to run child.");
                            #[cfg(target_os = "linux")]
                            kill_prefix(app).expect("Failed to kill wine prefix");
                            println!("binary exited with status: {}", output.status);
                        } else if let Err(e) = child {
                            panic!("Binary exited unexpectedly: {e}");
                        }
                    } else {
                        println!("Failed to fetch version.");
                    }
                }
                "kill" => {
                    #[cfg(target_os = "linux")]
                    kill_prefix(app).expect("Failed to kill wine prefix");
                }
                "winecfg" => {
                    #[cfg(target_os = "linux")]
                    {
                        get_wine(app)
                            .cmd()
                            .arg("winecfg")
                            .output()
                            .expect("Failed to run winecfg");
                    }
                }
                _ => todo!(),
            }
        } else {
            println!("No operation was specified, re-run with --help to receive the full list of options.")
        }
    }
}
