/// Servidor ID/Relay fixo (build Rech), aplicado antes de qualquer tela ler as opções.
/// `OVERWRITE_SETTINGS` tem prioridade sobre o salvo e bloqueia `Config::set_option()`.
pub fn set_fixed_server_config() {
    let mut overwrite = hbb_common::config::OVERWRITE_SETTINGS.write().unwrap();
    overwrite.insert(
        "custom-rendezvous-server".to_owned(),
        "remoto.rech.com.br".to_owned(),
    );
    overwrite.insert("relay-server".to_owned(), "remoto.rech.com.br".to_owned());
    overwrite.insert(
        "api-server".to_owned(),
        "https://remoto.rech.com.br".to_owned(),
    );
    overwrite.insert(
        "key".to_owned(),
        "5cnWVsqVGoLdAA+LORCxJ7iE6SIxFniJN0cStIASCos=".to_owned(),
    );
    drop(overwrite);
    hbb_common::config::BUILTIN_SETTINGS
        .write()
        .unwrap()
        .insert("hide-server-settings".to_owned(), "Y".to_owned());
    set_portable_defaults();
}

/// Build portable: as gravações vão para a pasta da máquina, ao lado da configuração.
/// Chamado em todo processo (UI e `--server`), senão cada um gravaria num lugar.
fn set_portable_defaults() {
    #[cfg(windows)]
    {
        let Some(dirs) = hbb_common::directories_next::BaseDirs::new() else {
            return;
        };
        let Some(dir) = portable_layout::current_machine_dir(dirs.config_dir()) else {
            return;
        };
        hbb_common::config::DEFAULT_LOCAL_SETTINGS
            .write()
            .unwrap()
            .insert(
                base::config::keys::OPTION_VIDEO_SAVE_DIRECTORY.to_owned(),
                dir.join(portable_layout::RECORDING_DIR)
                    .to_string_lossy()
                    .into_owned(),
            );
    }
}

#[cfg(all(test, windows))]
mod portable_tests {
    use hbb_common::config::{Config, LocalConfig};

    const CHILD_RAN: &str = "child-ran";

    // Only does something inside the copy made by `config_follows_the_portable_folder`.
    #[test]
    #[ignore]
    fn portable_child() {
        let exe = std::env::current_exe().unwrap();
        let Some(root) = portable_layout::root_from_exe(&exe) else {
            return;
        };
        let dirs = hbb_common::directories_next::BaseDirs::new().unwrap();
        let machine = portable_layout::current_machine_dir(dirs.config_dir()).unwrap();
        assert!(Config::path("").starts_with(machine.join("config")));
        assert_eq!(Config::log_path(), machine.join("log"));
        super::set_portable_defaults();
        assert_eq!(
            LocalConfig::get_option(base::config::keys::OPTION_VIDEO_SAVE_DIRECTORY),
            machine
                .join(portable_layout::RECORDING_DIR)
                .to_string_lossy()
                .into_owned()
        );
        std::fs::write(root.join(CHILD_RAN), b"").unwrap();
    }

    // The patched directories-next only reacts to the executable's own folder, so the check runs in a copy.
    #[test]
    fn config_follows_the_portable_folder() {
        let temp = std::env::temp_dir().join(format!("rech-portable-{}", std::process::id()));
        let root = temp.join("rustdesk_data");
        let app = root.join(portable_layout::APP_DIR);
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(root.join(portable_layout::MARKER), b"").unwrap();
        let exe = std::env::current_exe().unwrap();
        let copy = app.join(exe.file_name().unwrap());
        std::fs::copy(&exe, &copy).unwrap();

        let status = std::process::Command::new(&copy)
            .args(["--ignored", "--exact", "rech::portable_tests::portable_child"])
            .status();
        let child_ran = root.join(CHILD_RAN).is_file();
        let _ = std::fs::remove_dir_all(&temp);

        assert!(status.unwrap().success());
        assert!(child_ran, "portable_child did not run in the copied binary");
    }
}
