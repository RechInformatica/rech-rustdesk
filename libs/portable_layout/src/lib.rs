//! Folder layout of the Rech portable build, shared by the launcher, the patched
//! `directories-next` and the app:
//!
//! ```text
//! <launcher folder>\<app>_data\
//!     .portable          marker, empty
//!     app\               extraction, wiped on every new version
//!     <machine>\config\  Config::path()
//!     <machine>\log\     Config::log_path()
//! ```

use std::path::{Path, PathBuf};

pub const MARKER: &str = ".portable";
pub const APP_DIR: &str = "app";
pub const RECORDING_DIR: &str = "recording";
const DEFAULT_MACHINE_DIR: &str = "default";

/// The portable root of an executable extracted into `<root>\app\`.
pub fn root_from_exe(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    if !dir.file_name()?.to_str()?.eq_ignore_ascii_case(APP_DIR) {
        return None;
    }
    Some(dir.parent()?.to_path_buf())
}

/// The machine name becomes a folder name, so nothing in it may leave the root.
pub fn sanitize_machine_name(name: &str) -> String {
    let name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if name.is_empty() {
        DEFAULT_MACHINE_DIR.to_owned()
    } else {
        name
    }
}

/// SYSTEM and the other service accounts must not follow a marker in a folder the user controls.
pub fn is_service_profile(roaming: &Path) -> bool {
    let path = roaming.to_string_lossy().to_lowercase();
    path.contains("systemprofile") || path.contains("serviceprofiles")
}

/// The data folder of this machine, or `None` when the build is not running portable.
pub fn machine_dir(
    exe: &Path,
    roaming: &Path,
    machine_name: &str,
    marker_exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if is_service_profile(roaming) {
        return None;
    }
    let root = root_from_exe(exe)?;
    if !marker_exists(&root.join(MARKER)) {
        return None;
    }
    Some(root.join(sanitize_machine_name(machine_name)))
}

/// `machine_dir` for the running process. `roaming` is the real roaming AppData of the process.
#[cfg(windows)]
pub fn current_machine_dir(roaming: &Path) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    machine_dir(&exe, roaming, &computer_name()?, |marker| marker.is_file())
}

// The physical NetBIOS name does not change with the DNS domain, so the identity survives a network change.
#[cfg(windows)]
fn computer_name() -> Option<String> {
    use winapi::um::sysinfoapi::{ComputerNamePhysicalNetBIOS, GetComputerNameExW};

    let mut len = 0u32;
    unsafe { GetComputerNameExW(ComputerNamePhysicalNetBIOS, std::ptr::null_mut(), &mut len) };
    if len == 0 {
        return None;
    }
    let mut buf = vec![0u16; len as usize];
    if unsafe { GetComputerNameExW(ComputerNamePhysicalNetBIOS, buf.as_mut_ptr(), &mut len) } == 0
    {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe_in(dirs: &[&str]) -> PathBuf {
        let mut path = PathBuf::from("base");
        for dir in dirs {
            path.push(dir);
        }
        path.join("rustdesk.exe")
    }

    #[test]
    fn root_is_the_folder_above_app() {
        assert_eq!(
            root_from_exe(&exe_in(&["rustdesk_data", "app"])),
            Some(PathBuf::from("base").join("rustdesk_data"))
        );
        assert_eq!(
            root_from_exe(&exe_in(&["rustdesk_data", "APP"])),
            Some(PathBuf::from("base").join("rustdesk_data"))
        );
        assert_eq!(root_from_exe(&exe_in(&["Program Files", "RustDesk"])), None);
    }

    #[test]
    fn machine_name_cannot_leave_the_root() {
        assert_eq!(sanitize_machine_name("ESTACAO-01"), "ESTACAO-01");
        assert_eq!(sanitize_machine_name("..\\..\\Windows"), "Windows");
        assert_eq!(sanitize_machine_name("a/b:c"), "abc");
        assert_eq!(sanitize_machine_name(".."), "default");
        assert_eq!(sanitize_machine_name(""), "default");
    }

    #[test]
    fn service_profiles_are_detected_in_any_case() {
        for path in [
            r"C:\Windows\system32\config\systemprofile\AppData\Roaming",
            r"C:\WINDOWS\System32\Config\SystemProfile\AppData\Roaming",
            r"C:\Windows\ServiceProfiles\LocalService\AppData\Roaming",
            r"c:\windows\serviceprofiles\networkservice\appdata\roaming",
        ] {
            assert!(is_service_profile(Path::new(path)), "{path}");
        }
        assert!(!is_service_profile(Path::new(r"C:\Users\ana\AppData\Roaming")));
    }

    #[test]
    fn machine_dir_needs_the_marker_and_a_user_profile() {
        let exe = exe_in(&["rustdesk_data", "app"]);
        let user = Path::new(r"C:\Users\ana\AppData\Roaming");
        let system = Path::new(r"C:\Windows\system32\config\systemprofile\AppData\Roaming");
        let root = PathBuf::from("base").join("rustdesk_data");
        let marker = root.join(MARKER);
        let has_marker = |path: &Path| path == marker.as_path();

        assert_eq!(machine_dir(&exe, user, "PC1", has_marker), Some(root.join("PC1")));
        assert_eq!(machine_dir(&exe, user, "PC1", |_| false), None);
        assert_eq!(machine_dir(&exe, system, "PC1", has_marker), None);
        assert_eq!(
            machine_dir(&exe_in(&["RustDesk"]), user, "PC1", |_| true),
            None
        );
    }
}
