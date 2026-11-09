// Identificador técnico (serviço, exe, chaves de registro, esquema de URI): não pode ter espaço.
const APP_NAME: &str = "RechRustdesk";
const DISPLAY_NAME: &str = "Rech Rustdesk";

/// Nome para pastas visíveis (ex.: Program Files); só vale quando o app é o build Rech.
#[allow(dead_code)]
pub fn display_name() -> String {
    if hbb_common::config::APP_NAME.read().unwrap().as_str() == APP_NAME {
        DISPLAY_NAME.to_owned()
    } else {
        hbb_common::config::APP_NAME.read().unwrap().clone()
    }
}

/// Nome do app (build Rech)
/// Roda antes de `load_custom_client()`, que ainda pode sobrescrever. Deixa `is_custom_client()` true, o que desliga o auto-update oficial.
pub fn set_default_app_name() {
    *hbb_common::config::APP_NAME.write().unwrap() = APP_NAME.to_owned();
}

/// Servidor ID/Relay fixo (build Rech)
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

/// Processo que hospeda o servidor: `--server` ou o `--server-session` SYSTEM.
pub fn is_server_process() -> bool {
    #[cfg(windows)]
    if crate::rech_session_server::is_system_session_server() {
        return true;
    }
    crate::is_server()
}

/// Postfix do IPC principal; o servidor SYSTEM usa um pipe próprio para não colidir com o do usuário.
pub fn main_ipc_postfix() -> String {
    #[cfg(windows)]
    if crate::rech_session_server::is_system_session_server() {
        return crate::rech_session_server::system_ipc_postfix();
    }
    String::new()
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
    match hbb_common::config::LocalConfig::get_option(OPTION_SYSTEM_SESSION_SERVER).as_str() {
        "N" => false,
        "Y" => true,
        _ => crate::get_builtin_option(OPTION_SYSTEM_SESSION_SERVER) == "Y",
    }
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
const HANDOFF_FILES: [&str; 2] = ["session_config.toml", "session_config2.toml"];
#[cfg(windows)]
const HANDOFF_STOP_FILE: &str = "stop";

#[cfg(windows)]
/// Pasta do handoff, dentro da config do próprio usuário (privada pelo ACL do perfil).
pub fn session_handoff_dir() -> hbb_common::ResultType<std::path::PathBuf> {
    let base = hbb_common::config::Config::file()
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| hbb_common::anyhow::anyhow!("config dir unavailable"))?;
    let dir = base.join(format!(
        "{}_{}",
        SESSION_CONFIG_SUBDIR,
        crate::platform::windows::get_current_process_session_id().unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(windows)]
fn remove_if_exists(path: &std::path::Path) -> hbb_common::ResultType<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => hbb_common::bail!("Failed to remove '{}': {}", path.display(), e),
    }
}

#[cfg(windows)]
/// Grava a config do usuário (ID, chaves, senha) em `dir` para o servidor SYSTEM importar.
/// Falha se não conseguir recriar os arquivos, para não importar conteúdo de terceiros.
pub fn export_session_config_for_system(dir: &std::path::Path) -> hbb_common::ResultType<()> {
    // O servidor SYSTEM precisa do mesmo ID e do mesmo par de chaves do usuário.
    let _ = hbb_common::config::Config::get_id();
    let _ = hbb_common::config::Config::get_key_pair();
    remove_if_exists(&dir.join(HANDOFF_STOP_FILE))?;
    for name in HANDOFF_FILES {
        remove_if_exists(&dir.join(name))?;
    }
    let cfg = dir.join(HANDOFF_FILES[0]);
    let cfg2 = dir.join(HANDOFF_FILES[1]);
    hbb_common::config::store_path(cfg.clone(), hbb_common::config::Config::get())?;
    hbb_common::config::store_path(cfg2.clone(), hbb_common::config::Config2::get())?;
    for file in [&cfg, &cfg2] {
        if let Err(err) =
            crate::platform::windows::set_path_permission_for_portable_service_shmem_file(file)
        {
            wipe_session_config(dir, false);
            return Err(err);
        }
    }
    Ok(())
}

#[cfg(windows)]
/// Importa a config do handoff e apaga os arquivos temporários.
pub fn import_and_wipe_session_config(dir: &std::path::Path) {
    let cfg = hbb_common::config::load_path::<hbb_common::config::Config>(dir.join(HANDOFF_FILES[0]));
    if !cfg.is_empty() {
        hbb_common::config::Config::set(cfg);
    }
    let cfg2 =
        hbb_common::config::load_path::<hbb_common::config::Config2>(dir.join(HANDOFF_FILES[1]));
    hbb_common::config::Config2::set(cfg2);
    wipe_session_config(dir, false);
}

#[cfg(windows)]
/// Remove os arquivos do handoff; `include_stop` remove também o sinal de parada.
pub fn wipe_session_config(dir: &std::path::Path, include_stop: bool) {
    for name in HANDOFF_FILES {
        let _ = std::fs::remove_file(dir.join(name));
    }
    if include_stop {
        let _ = std::fs::remove_file(dir.join(HANDOFF_STOP_FILE));
    }
}

#[cfg(windows)]
/// O app portable pede ao servidor SYSTEM que encerre.
pub fn request_session_server_stop(dir: &std::path::Path) {
    let _ = std::fs::File::create(dir.join(HANDOFF_STOP_FILE));
}

#[cfg(windows)]
pub fn session_server_stop_requested(dir: &std::path::Path) -> bool {
    dir.join(HANDOFF_STOP_FILE).exists()
}

/// Versão Rech do build
/// Vazia em build local, de PR ou feito dentro de container (Linux), onde a variável não chega.
pub fn get_rech_version() -> String {
    let tag = option_env!("TAG_NAME").unwrap_or_default();
    if is_rech_version_tag(tag) {
        tag.to_owned()
    } else {
        String::new()
    }
}

/// Texto da versão na tela "Sobre": `Rech Rustdesk 1.5.1-r42 (RustDesk 1.5.1)`
#[allow(dead_code)]
pub fn get_display_version() -> String {
    format_display_version(&get_rech_version(), crate::VERSION)
}

fn format_display_version(rech: &str, base: &str) -> String {
    if rech.is_empty() {
        base.to_owned()
    } else {
        format!("{} {} (RustDesk {})", DISPLAY_NAME, rech, base)
    }
}

fn is_rech_version_tag(tag: &str) -> bool {
    let Some((base, n)) = tag.rsplit_once("-r") else {
        return false;
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let parts: Vec<&str> = base.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| digits(p)) && digits(n)
}

#[cfg(test)]
mod tests {
    use super::{format_display_version, is_rech_version_tag};

    #[test]
    fn rech_version_tag() {
        assert!(is_rech_version_tag("1.5.1-r42"));
        assert!(is_rech_version_tag("10.0.12-r1"));
        assert!(!is_rech_version_tag(""));
        assert!(!is_rech_version_tag("nightly"));
        assert!(!is_rech_version_tag("1.5.1"));
        assert!(!is_rech_version_tag("1.5.1-rech.3"));
        assert!(!is_rech_version_tag("1.5.1-r"));
        assert!(!is_rech_version_tag("1.5-r4"));
        assert!(!is_rech_version_tag("v1.5.1-r4"));
    }

    #[test]
    fn display_version() {
        assert_eq!(format_display_version("", "1.5.1"), "1.5.1");
        assert_eq!(
            format_display_version("1.5.1-r42", "1.5.1"),
            "Rech Rustdesk 1.5.1-r42 (RustDesk 1.5.1)"
        );
    }
}
