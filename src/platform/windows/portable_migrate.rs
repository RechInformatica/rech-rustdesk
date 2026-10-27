use hbb_common::{config::Config, directories_next::BaseDirs, log};
use std::{io, path::Path};

/// Installing from the portable build: `--import-config` already moves the ID, keys and options
/// into the service, so this copies what lives in the user's AppData in the installed build.
pub(super) fn copy_user_config() {
    let Some(base) = BaseDirs::new() else {
        return;
    };
    // BaseDirs is not patched, so this is the real AppData the installed app will read.
    if portable_layout::current_machine_dir(base.config_dir()).is_none() {
        return;
    }
    let app_name = crate::get_app_name();
    let src = Config::path("");
    let dst = base.config_dir().join(&app_name).join("config");
    for name in [
        "peers".to_owned(),
        format!("{app_name}_local.toml"),
        format!("{app_name}_default.toml"),
    ] {
        if let Err(err) = copy_missing(&src.join(&name), &dst.join(&name)) {
            log::warn!("Failed to copy portable {name} to {}: {err}", dst.display());
        }
    }
}

// Never overwrites: an earlier installation keeps its own data.
fn copy_missing(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                copy_missing(&entry.path(), &dst.join(entry.file_name()))?;
            }
        }
    } else if src.is_file() && !dst.exists() {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dst)?;
    }
    Ok(())
}
