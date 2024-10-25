use crate::client_settings::ClientDeployment;

#[derive(serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub checksum: String,
    pub size: i64,
    pub zipsize: i64,
}

impl Package {
    pub fn get_extraction_dir(&self) -> String {
        let res = match self.name.as_str() {
            "RobloxApp.zip" => ".",
            "ApplicationConfig.zip" => "ApplicationConfig",
            "BuiltInPlugins.zip" => "BuiltInPlugins",
            "BuiltInStandalonePlugins.zip" => "BuiltInStandalonePlugins",
            "Plugins.zip" => "Plugins",
            "Qml.zip" => "Qml",
            "RibbonConfig.zip" => "RibbonConfig",
            "StudioFonts.zip" => "StudioFonts",
            "WebView2.zip" => ".",
            "WebView2RuntimeInstaller.zip" => "WebView2RuntimeInstaller",
            "RobloxStudio.zip" => ".",
            "Libraries.zip" => ".",
            "LibrariesQt5.zip" => ".",
            "content-avatar.zip" => "content/avatar",
            "content-configs.zip" => "content/configs",
            "content-fonts.zip" => "content/fonts",
            "content-models.zip" => "content/models",
            "content-qt_translations.zip" => "content/qt_translations",
            "content-sky.zip" => "content/sky",
            "content-sounds.zip" => "content/sounds",
            "shaders.zip" => "shaders",
            "ssl.zip" => "ssl",
            "content-textures2.zip" => "content/textures",
            "content-textures3.zip" => "PlatformContent/pc/textures",
            "content-studio_svg_textures.zip" => "content/studio_svg_textures",
            "content-terrain.zip" => "PlatformContent/pc/terrain",
            "content-platform-fonts.zip" => "PlatformContent/pc/fonts",
            "extracontent-places.zip" => "ExtraContent/places",
            "content-api-docs.zip" => "content/api_docs",
            "extracontent-scripts.zip" => "ExtraContent/scripts",
            "extracontent-luapackages.zip" => "ExtraContent/LuaPackages",
            "extracontent-translations.zip" => "ExtraContent/translations",
            "extracontent-models.zip" => "ExtraContent/models",
            "extracontent-textures.zip" => "ExtraContent/textures",
            "redist.zip" => ".",
            _ => todo!()
        };

        res.into()
    }
}

fn parse_rbx_pkg_manifest(manifest: &str) -> Vec<Package> {
    let mut packages: Vec<Package> = Vec::new();
    let package_manifests: Vec<&str> = manifest.split("\r\n").collect();
    let package_count = package_manifests.iter().count();
    // FIXME: i dont like this too much
    let mut i = 0;
    while i < package_count {
        let package = *package_manifests
            .get(i)
            .expect(format!("Failed to get package at index {}", i).as_str());
        if i == 0 {
            if package != "v0" {
                println!("Invalid manifest????????");
            }
            i += 1;
            continue;
        } else if i >= package_count - 1 {
            // roblox sends us a newline, so we have to skip it
            i += 1;
            continue;
        }

        let checksum = *package_manifests.get(i + 1).unwrap();
        let size = *package_manifests.get(i + 2).unwrap();
        let zipsize = *package_manifests.get(i + 3).unwrap();

        packages.push(Package {
            name: package.into(),
            checksum: checksum.into(),
            size: size.parse().unwrap(),
            zipsize: zipsize.parse().unwrap(),
        });

        i += 4;
    }

    packages
}

pub async fn get_mirror() -> Result<String, String> {
    let mirrors = [
        "https://setup.rbxcdn.com",
        "https://roblox-setup.cachefly.net",
        "https://s3.amazonaws.com/setup.roblox.com",
    ];

    for mirror in mirrors {
        let res = reqwest::get(mirror).await;
        if res.is_ok() {
            return Ok(mirror.into());
        }
    }

    Err("Couldn't find a valid mirror".into())
}

pub async fn get_mirror_packages(
    mirror: &str,
    deployment: &ClientDeployment,
) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let url = format!(
        "{mirror}/{}-rbxPkgManifest.txt",
        deployment.client_version_upload
    );
    println!("Fetching from URL {url}");
    let res = reqwest::get(url.as_str()).await?.text().await?;
    Ok(parse_rbx_pkg_manifest(res.as_str()))
}
