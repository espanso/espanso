use log::info;
use std::os::unix::fs::symlink;

const MODULO_APP_BUNDLE_NAME: &str = "Modulo.app";
const MODULO_APP_BUNDLE_PLIST_CONTENT: &'static str = include_str!("../../res/mac/modulo.plist");

pub fn generate_modulo_app_bundle(modulo_path: &str) -> Result<PathBuf, std::io::Error> {
    let data_dir = crate::context::get_data_dir();

    let modulo_app_dir = data_dir.join(MODULO_APP_BUNDLE_NAME);

    // Remove previous bundle if present
    if modulo_app_dir.exists() {
        std::fs::remove_dir_all(&modulo_app_dir)?;
    }

    // Recreate the App bundle stub
    std::fs::create_dir(&modulo_app_dir)?;

    let contents_dir = modulo_app_dir.join("Contents");
    std::fs::create_dir(&contents_dir)?;

    let macos_dir = contents_dir.join("MacOS");
    std::fs::create_dir(&macos_dir)?;

    // Generate the Plist file
    let plist_content = MODULO_APP_BUNDLE_PLIST_CONTENT.replace("{{{modulo_path}}}", modulo_path);
    let plist_file = contents_dir.join("Info.plist");
    std::fs::write(plist_file, plist_content)?;

    // Generate the symbolic link to the modulo binary
    let target_link = macos_dir.join("modulo");
    symlink(modulo_path, &target_link)?;

    info!("Created Modulo APP stub at: {:?}", &target_link);

    Ok(target_link)
}