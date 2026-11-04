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
    let mut builtin = hbb_common::config::BUILTIN_SETTINGS.write().unwrap();
    builtin.insert("hide-server-settings".to_owned(), "Y".to_owned());
    builtin.insert("rech-system-session-server".to_owned(), "Y".to_owned());
}

const OPTION_SYSTEM_SESSION_SERVER: &str = "rech-system-session-server";

/// Motivo de fechamento enviado ao cliente quando o servidor SYSTEM assume a sessão.
pub const SYSTEM_HANDOVER_REASON: &str = "rech-system-handover";

/// Postfix do IPC principal; o servidor SYSTEM usa um pipe próprio para não colidir com o do usuário.
pub fn main_ipc_postfix() -> &'static str {
    #[cfg(windows)]
    if crate::rech_session_server::is_system_session_server() {
        return crate::rech_session_server::SYSTEM_IPC_POSTFIX;
    }
    ""
}

/// Motivo de fechamento do StopService: sinaliza handover quando o servidor SYSTEM existe.
pub fn stop_service_close_reason() -> &'static str {
    #[cfg(windows)]
    if crate::rech_session_server::system_session_server_running() {
        return SYSTEM_HANDOVER_REASON;
    }
    ""
}

/// True quando o motivo foi tratado (tela de reinício exibida) e o chamador deve encerrar.
pub fn handle_close_reason<T: crate::ui_session_interface::InvokeUiSession>(
    handler: &crate::ui_session_interface::Session<T>,
    reason: &str,
) -> bool {
    if reason == SYSTEM_HANDOVER_REASON {
        handler.lc.write().unwrap().mark_restarting_remote_device();
    }
    if !handler.is_restarting_remote_device() {
        return false;
    }
    crate::client::Interface::msgbox(
        handler,
        "restarting",
        "Restarting remote device",
        "Connection in progress. Please wait.",
        "",
    );
    true
}

/// O servidor SYSTEM assume a sessão e fecha a conexão; o cliente reconecta sozinho.
pub fn on_elevation_started<T: crate::ui_session_interface::InvokeUiSession>(
    handler: &crate::ui_session_interface::Session<T>,
    elevation_requested: bool,
) {
    if elevation_requested {
        handler.lc.write().unwrap().mark_restarting_remote_device();
    }
}

pub fn on_portable_service_running<T: crate::ui_session_interface::InvokeUiSession>(
    handler: &crate::ui_session_interface::Session<T>,
    running: bool,
) {
    if running {
        handler.lc.write().unwrap().clear_restarting_remote_device();
    }
}
#[cfg(windows)]
const SESSION_CONFIG_SUBDIR: &str = "rech_session_server";

/// Liga/desliga o modo "servidor SYSTEM dono da sessão" no portable.
/// Enquanto desligado, todo o resto do plano fica inerte.
pub fn use_system_session_server() -> bool {
    hbb_common::config::LocalConfig::get_option(OPTION_SYSTEM_SESSION_SERVER) == "Y"
        || crate::get_builtin_option(OPTION_SYSTEM_SESSION_SERVER) == "Y"
}

/// True quando rodando como payload portable (não instalado).
/// O gate não deve disparar na versão instalada.
pub fn is_portable_runtime() -> bool {
    #[cfg(windows)]
    {
        !crate::platform::is_installed()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(windows)]
/// Caminho determinístico do arquivo de handoff de config (`.toml`).
pub fn session_config_path() -> hbb_common::ResultType<std::path::PathBuf> {
    let dir = crate::platform::user_accessible_folder()?;
    let dir = dir.join(hbb_common::config::APP_NAME.read().unwrap().clone());
    let dir = dir.join(SESSION_CONFIG_SUBDIR);
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("session_config.toml"))
}

#[cfg(windows)]
fn session_config2_path(cfg_path: &std::path::Path) -> std::path::PathBuf {
    std::path::PathBuf::from(cfg_path.to_string_lossy().replace(".toml", "2.toml"))
}

#[cfg(windows)]
/// Grava a config do usuário logado (ID/senha/permissões) num local acessível ao SYSTEM
/// e ao usuário, para o servidor SYSTEM transitório importar. Retorna o caminho `.toml`.
pub fn export_session_config_for_system() -> hbb_common::ResultType<std::path::PathBuf> {
    let cfg_path = session_config_path()?;
    // O servidor SYSTEM precisa do mesmo ID e do mesmo par de chaves do usuário.
    let _ = hbb_common::config::Config::get_id();
    let _ = hbb_common::config::Config::get_key_pair();
    let cfg2_path = session_config2_path(&cfg_path);
    hbb_common::config::store_path(cfg_path.clone(), hbb_common::config::Config::get())?;
    hbb_common::config::store_path(cfg2_path, hbb_common::config::Config2::get())?;
    Ok(cfg_path)
}

#[cfg(windows)]
/// Importa a config do handoff e apaga os arquivos temporários.
pub fn import_and_wipe_session_config(path: &std::path::Path) {
    let cfg2_path = session_config2_path(path);
    let cfg = hbb_common::config::load_path::<hbb_common::config::Config>(path.to_path_buf());
    if !cfg.is_empty() {
        hbb_common::config::Config::set(cfg);
    }
    let cfg2 = hbb_common::config::load_path::<hbb_common::config::Config2>(cfg2_path.clone());
    hbb_common::config::Config2::set(cfg2);
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(cfg2_path);
}

#[cfg(windows)]
/// Remove os arquivos temporários do handoff sem importar (limpeza de saída).
pub fn wipe_session_config() {
    if let Ok(path) = session_config_path() {
        let cfg2_path = session_config2_path(&path);
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(cfg2_path);
    }
}
