use std::{
    fs,
    path::PathBuf,
    process::{Child, Stdio},
};

use bytes::Bytes;
use pe_parser::pe::parse_portable_executable;

use crate::app_data::{ensure_prefix_exists, get_webview_installer_dir, get_wineroot_string};

pub fn run_windows_binary(binary_file: PathBuf, app_name: &String) -> Result<Child, String> {
    println!("Running {}", binary_file.to_str().unwrap());
    #[cfg(target_os = "windows")]
    {
        let child = std::process::Command::new(binary_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        return Ok(child);
    }
    let prefix_path = ensure_prefix_exists(&app_name);
    let wine = winers::Wine::new(prefix_path.to_str().unwrap(), get_wineroot_string());
    if let Err(e) = wine.init() {
        panic!("Error initializing wine: {e}");
    }
    let mut cmd = wine.cmd();
    cmd.arg(binary_file);
    let child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    Ok(child)
}

pub async fn download_webview2(webview_installer_url: &str) -> Result<Bytes, String> {
    if get_webview_installer_dir().exists() {
        let data = fs::read(get_webview_installer_dir()).unwrap();
        return Ok(Bytes::from(data));
    }

    let res = reqwest::get(webview_installer_url)
        .await
        .unwrap()
        .bytes()
        .await;

    if let Ok(res) = res {
        fs::write(get_webview_installer_dir(), res.slice(0..res.len()))
            .expect("Failed to cache webview!");
        Ok(res)
    } else {
        Err("Failed to download webview.".into())
    }
}

fn install_webview_wine(app_name: &String, binary_data: Bytes) -> Result<(), String> {
    run_windows_binary(get_webview_installer_dir(), &app_name).unwrap();

    // TODO
    /*
    let pe = parse_portable_executable(&binary_data).unwrap();
    pe.section_table.iter().for_each(|hdr| {
        if let Some(hdr_name) = hdr.get_name() {
            let search = ".rsrc\0\0\0";
            println!("Header: {:?}, search: {:?}", hdr_name, search);
            if hdr_name.as_str() == search {
                println!("Resource header found.");
                let ptr: *const u32 = &hdr.pointer_to_raw_data;
                for i in 0..hdr.size_of_raw_data {
                    let cnt: usize = if let Ok(cnt) = i.try_into() {
                        cnt
                    } else {
                        continue;
                    };
                    let curr_ptr = unsafe { ptr.add(cnt) };
                    let value = unsafe { *curr_ptr };
                }
            }
        }
    });
    */
    Ok(())
}

pub async fn install_webview2(app_name: &String) -> Result<(), String> {
    let webview_installer_url = "https://catalog.s.download.windowsupdate.com/c/msdownload/update/software/updt/2023/09/microsoftedgestandaloneinstallerx64_1c890b4b8dd6b7c93da98ebdc08ecdc5e30e50cb.exe";

    let res = download_webview2(&webview_installer_url).await;
    if let Ok(res) = res {
        #[cfg(target_os = "windows")]
        {
            if run_windows_binary(get_webview_installer_dir(), app_name).is_ok() {
                return Ok(());
            } else {
                return Err("Failed to execute webview binary".into());
            }
        }
        install_webview_wine(&app_name, res)
    } else {
        Err("Failed to download webview installer.".into())
    }
}
