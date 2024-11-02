use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering
    }
};

use clap::{arg, command};
use common::{
    app_data::{get_binary_name, get_binary_type, get_wine, kill_prefix},
    client_settings::get_client_version,
    download::{download_package, format_file_size, install_package, write_app_settings_xml},
    flog::begin_flog_watch,
    mirror::{get_mirror, get_mirror_packages},
    runner::{install_webview2, run_windows_binary},
};

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(arg!([app] "Roblox app to operate on").value_parser(["player", "studio"]))
        .arg(
            arg!([operation] "Operation to run")
                .value_parser(["run", "kill", "winecfg", "regedit", "delete"]),
        )
        .get_matches();

    if let Some(app) = matches.get_one::<String>("app") {
        let binary_type = get_binary_type(app);
        let binary_name = get_binary_name(app);

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

                        // Add CTRL+C handler on debug builds to kill the Roblox process
                        #[cfg(debug_assertions)]
                        {
                            let app_clone = app.clone();
                            let running = Arc::new(AtomicBool::new(true));
                            let r = running.clone();
                            ctrlc::set_handler(move || {
                                r.store(false, Ordering::SeqCst);
                                kill_prefix(&app_clone).expect(format!("Failed to kill {}", &app_clone).as_str());
                            }).expect("Error setting Ctrl-C handler");
                        }

                        // we need to keep watcher in scope, or it gets deleted and stops working
                        let _watcher = begin_flog_watch(app);
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
                    kill_prefix(app).expect(format!("Failed to kill {}", app).as_str());
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
                "regedit" => {
                    #[cfg(target_os = "linux")]
                    {
                        get_wine(app)
                            .cmd()
                            .arg("regedit")
                            .output()
                            .expect("Failed to run regedit");
                    }
                }
                "delete" => {
                    #[cfg(target_os = "linux")]
                    {
                        get_wine(app).delete().unwrap();
                    }
                }
                _ => println!("Operation specified was invalid, re-run with --help to receive the full list of operations."),
            }
        } else {
            println!("No operation was specified, re-run with --help to receive the full list of operations.")
        }
    }
}
