use std::path::{Component, Path, PathBuf, Prefix};

/// Extraction folder of the Rech portable build, `<launcher folder>\<app>_data\app`, or `None` to
/// keep the stock `%LOCALAPPDATA%` extraction.
pub fn extract_dir(payload_exe: &str, click_setup: bool, args: &[String]) -> Option<PathBuf> {
    // Installing or updating must not leave a data folder beside the installer or in %TEMP%.
    if click_setup
        || args.iter().any(|arg| arg == "--silent-install")
        || args.first().is_some_and(|arg| arg == "--update")
    {
        return None;
    }
    let launcher = std::env::current_exe().ok()?;
    let root = portable_root(&launcher, &super::app_dir_name(payload_exe))?;
    Some(root.join(portable_layout::APP_DIR))
}

// Named after the payload, not the launcher, whose name changes with every version.
fn portable_root(launcher: &Path, app: &str) -> Option<PathBuf> {
    let dir = launcher.parent()?;
    let Some(letter) = drive_letter(dir) else {
        println!("portable: {} is not on a local drive, using AppData", dir.display());
        return None;
    };
    // The portable service runs as SYSTEM, which cannot reach the user's network drives.
    if !is_local_drive(letter) {
        println!("portable: drive {letter}: is not local, using AppData");
        return None;
    }
    let root = dir.join(format!("{app}_data"));
    if let Err(err) = prepare_root(&root) {
        println!("portable: {} is not writable ({err}), using AppData", root.display());
        return None;
    }
    Some(root)
}

fn drive_letter(path: &Path) -> Option<char> {
    match path.components().next()? {
        Component::Prefix(prefix) => match prefix.kind() {
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => Some(letter as char),
            _ => None,
        },
        _ => None,
    }
}

fn is_local_drive(letter: char) -> bool {
    use windows::{core::HSTRING, Win32::Storage::FileSystem::GetDriveTypeW};
    use winapi::um::winbase::{DRIVE_FIXED, DRIVE_REMOVABLE};
    let kind = unsafe { GetDriveTypeW(&HSTRING::from(format!("{letter}:\\"))) };
    kind == DRIVE_FIXED || kind == DRIVE_REMOVABLE
}

fn prepare_root(root: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(root)?;
    let probe = root.join(".write-test");
    std::fs::write(&probe, b"")?;
    std::fs::remove_file(&probe)?;
    let marker = root.join(portable_layout::MARKER);
    if !marker.is_file() {
        std::fs::write(marker, b"")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_drive_letter_paths_are_portable() {
        assert_eq!(drive_letter(Path::new(r"C:\Users\ana\Downloads")), Some('C'));
        assert_eq!(drive_letter(Path::new(r"\\?\D:\tools")), Some('D'));
        assert_eq!(drive_letter(Path::new(r"\\server\share\tools")), None);
        assert_eq!(drive_letter(Path::new(r"\\?\UNC\server\share")), None);
    }
}
