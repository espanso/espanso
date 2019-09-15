// This functions are used to check if the required dependencies are satisfied
// before starting espanso

#[cfg(target_os = "linux")]
pub fn check_dependencies() -> bool {
    use std::process::Command;

    let mut result = true;

    // Make sure notify-send is installed
    let status = Command::new("notify-send")
        .arg("-v")
        .output();
    if let Err(_) = status {
        println!("Error: 'notify-send' command is needed for espanso to work correctly, please install it.");
        result = false;
    }

    // Make sure xclip is installed
    let status = Command::new("xclip")
        .arg("-version")
        .output();
    if let Err(_) = status {
        println!("Error: 'xclip' command is needed for espanso to work correctly, please install it.");
        result = false;
    }

    result
}

#[cfg(target_os = "macos")]
pub fn check_dependencies() -> bool {
    // TODO: check accessibility
}

#[cfg(target_os = "windows")]
pub fn check_dependencies() -> bool {
    // Nothing needed on windows
    true
}