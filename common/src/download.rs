use std::{
    fs,
    io::{self, Write},
};

use bytes::Bytes;
use zip::ZipArchive;

use crate::{client_settings::ClientDeployment, mirror::Package};

pub async fn download_package(
    mirror: &str,
    deployment: &ClientDeployment,
    package: &Package,
) -> Result<Bytes, reqwest::Error> {
    let url = format!(
        "{mirror}/{}-{}",
        deployment.client_version_upload, package.name
    );
    let data = reqwest::get(url).await?.bytes().await?;

    Ok(data)
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
