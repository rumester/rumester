use std::{
    fs,
    io::{self, Write},
};

use bytes::Bytes;
use zip::ZipArchive;

use crate::{app_data::get_package_dir, client_settings::ClientDeployment, mirror::Package};

pub fn format_file_size(size: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2}{}", size, UNITS[unit_index])
}

pub async fn download_package(
    mirror: &str,
    deployment: &ClientDeployment,
    package: &Package,
) -> Result<Bytes, String> {
    let package_dir = get_package_dir(&package);
    if let Ok(existing) = fs::read(&package_dir) {
        return Ok(Bytes::from(existing));
    }

    let url = format!(
        "{mirror}/{}-{}",
        deployment.client_version_upload, package.name
    );
    let res = reqwest::get(url).await;
    match res {
        Ok(res) => match res.bytes().await {
            Ok(data) => {
                if let Err(e) = fs::write(&package_dir, data.to_vec()) {
                    return Err(format!("Failed to write file: {e}"));
                }
                Ok(data)
            }
            Err(e) => Err(format!("Failed to get data: {e}")),
        },
        Err(e) => Err(format!("Failed to download package: {e}")),
    }
}

// TODO: needs better error handling
pub fn install_package(
    package: &Package,
    deployment: &ClientDeployment,
    data: &Bytes,
) -> Result<(), io::Error> {
    let mut archive = ZipArchive::new(io::Cursor::new(data)).expect("Failed to create ziparchive");
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Failed to find file by index");
        let p: String = file.name().replace("\\", "/");
        if p.ends_with("/") {
            continue;
        }
        let output_path = &deployment
            .get_install_dir()
            .join(package.get_extraction_dir())
            .join(p);

        if !output_path.parent().unwrap().exists() {
            if let Err(e) = fs::create_dir_all(output_path.parent().unwrap()) {
                return Err(e);
            }
        }
        let mut out = fs::File::create(output_path).expect("Failed to create file.");
        if let Err(e) = io::copy(&mut file, &mut out) {
            return Err(e);
        }
    }

    Ok(())
}

pub fn write_app_settings_xml(deployment: &ClientDeployment) {
    let mut file = fs::File::create(deployment.get_install_dir().join("AppSettings.xml")).unwrap();
    let mut content = String::new();
    content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n");
    content.push_str("<Settings>\r\n");
    content.push_str("        <ContentFolder>content</ContentFolder>\r\n");
    content.push_str("        <BaseUrl>http://www.roblox.com</BaseUrl>\r\n");
    content.push_str("</Settings>\r\n");
    file.write(content.as_bytes()).unwrap();
}
