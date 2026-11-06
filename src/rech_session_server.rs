use hbb_common::{log, tokio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use winapi::{
    shared::{
        minwindef::{DWORD, FALSE, TRUE},
        ntdef::NULL,
        winerror::ERROR_ACCESS_DENIED,
    },
    um::{
        handleapi::CloseHandle,
        minwinbase::STILL_ACTIVE,
        processthreadsapi::GetExitCodeProcess,
        synchapi::{CreateMutexW, OpenMutexW},
        winnt::{HANDLE, SYNCHRONIZE},
    },
};

/// True only in the `--server-session` process, i.e. when this process really is the
/// SYSTEM server that owns the session. The gate option alone must not change the
/// behavior of the portable app's own server.
static IS_SYSTEM_SESSION_SERVER: AtomicBool = AtomicBool::new(false);

fn current_session() -> u32 {
    crate::platform::windows::get_current_process_session_id().unwrap_or(0)
}

fn session_mutex_name() -> Vec<u16> {
    format!("Global\\{}_system_session_{}", crate::get_app_name(), current_session())
        .encode_utf16()
        .chain(Some(0))
        .collect()
}

pub fn mark_system_session_server() {
    IS_SYSTEM_SESSION_SERVER.store(true, Ordering::SeqCst);
    // O handle vive até o processo sair; o ACL padrão nega abertura ao usuário, mas OpenMutex distingue "negado" de "inexistente".
    unsafe {
        CreateMutexW(std::ptr::null_mut(), FALSE, session_mutex_name().as_ptr());
    }
}

pub fn is_system_session_server() -> bool {
    IS_SYSTEM_SESSION_SERVER.load(Ordering::SeqCst)
}

const SUPERVISE_INTERVAL_SECS: f32 = 1.0;
const NO_CONNECTION_TIMEOUT: Duration = Duration::from_secs(300);
const REMOTE_DROP_GRACE: Duration = Duration::from_secs(30);

/// Pipe próprio do servidor SYSTEM, por sessão; o ACL padrão de um pipe criado por SYSTEM bloqueia escrita do usuário.
pub fn system_ipc_postfix() -> String {
    format!("_system_session_{}", current_session())
}

fn server_exited(h: HANDLE) -> bool {
    if h.is_null() {
        return true;
    }
    unsafe {
        let mut exit_code: DWORD = 0;
        GetExitCodeProcess(h, &mut exit_code) == TRUE && exit_code != STILL_ACTIVE
    }
}

fn close_handle(h: HANDLE) {
    if !h.is_null() {
        unsafe {
            CloseHandle(h);
        }
    }
}

fn spawn_session_server(session_id: u32, dir: &std::path::Path) -> HANDLE {
    log::info!("launching system session server in session {}", session_id);
    match spawn_session_server_as_system(session_id, dir) {
        Ok(h) => h,
        Err(err) => {
            log::error!("Failed to launch system session server: {}", err);
            NULL
        }
    }
}

/// Lança o `--server-session` como SYSTEM na sessão interativa informada.
fn spawn_session_server_as_system(
    session_id: DWORD,
    dir: &std::path::Path,
) -> hbb_common::ResultType<HANDLE> {
    let cmd = format!(
        "\"{}\" --server-session {}{}",
        std::env::current_exe()?.to_str().unwrap_or(""),
        DIR_ARG_PREFIX,
        hex_encode(&dir.to_string_lossy()),
    );
    crate::platform::windows::launch_privileged_process(session_id, &cmd)
}

const DIR_ARG_PREFIX: &str = "--server-session-dir=";

fn hex_encode(s: &str) -> String {
    s.bytes().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Option<String> {
    if s.len() % 2 != 0 {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect();
    String::from_utf8(bytes?).ok()
}

/// Diretório de handoff repassado pela cadeia de elevação.
fn handoff_dir_from_args() -> Option<std::path::PathBuf> {
    std::env::args()
        .find_map(|arg| hex_decode(arg.strip_prefix(DIR_ARG_PREFIX)?))
        .map(std::path::PathBuf::from)
}

fn enabled_for_elevation() -> bool {
    crate::rech::use_system_session_server()
        && crate::rech::is_portable_runtime()
        && !crate::portable_service::has_portable_service_shmem_arg()
}

fn dir_arg() -> Option<String> {
    let dir = handoff_dir_from_args()?;
    Some(format!("{}{}", DIR_ARG_PREFIX, hex_encode(&dir.to_string_lossy())))
}

/// A pasta do handoff vem do processo do usuário; o UAC pode rodar sob outra conta e não a recalcula.
pub fn extend_elevate_arg(arg: &mut String) {
    if enabled_for_elevation() && !crate::platform::windows::is_root() {
        if let Some(dir_arg) = dir_arg() {
            arg.push(' ');
            arg.push_str(&dir_arg);
        }
    }
}

/// run_as_system herda a sessão do winlogon, então a sessão de quem pediu vai por argumento.
pub fn extend_run_as_system_arg(arg: &mut String) {
    if enabled_for_elevation() && !crate::platform::windows::is_root() {
        if let (Some(session_arg), Some(dir_arg)) = (session_arg(), dir_arg()) {
            arg.push(' ');
            arg.push_str(&session_arg);
            arg.push(' ');
            arg.push_str(&dir_arg);
        }
    }
}

/// Roda o supervisor quando o modo está ligado; false deixa o fluxo original seguir.
pub fn supervise_from_args() -> bool {
    if !enabled_for_elevation() {
        return false;
    }
    match (session_id_from_args(), handoff_dir_from_args()) {
        (Some(session_id), Some(dir)) => supervise(session_id, dir),
        _ => log::error!("Missing session id or handoff dir for system session server"),
    }
    true
}

/// Ponto de entrada do processo `--server-session`.
pub fn run_server_session() {
    log::info!("start --server-session as system session server");
    mark_system_session_server();
    let Some(dir) = handoff_dir_from_args() else {
        log::error!("Missing handoff dir for system session server");
        return;
    };
    crate::rech::import_and_wipe_session_config(&dir);
    std::thread::spawn(move || watch_remote_connections(dir));
    crate::start_server(true, false);
}

pub fn override_log_name(log_name: &mut String) {
    if session_id_from_args().is_some() {
        *log_name = "server-session".to_owned();
    }
}

/// Atende o pedido de elevação do cliente; None deixa o fluxo original seguir.
pub fn handle_elevation_request() -> Option<String> {
    if !crate::rech::use_system_session_server()
        || crate::platform::is_installed()
        || crate::portable_service::client::running()
    {
        return None;
    }
    if system_session_server_running() {
        return Some("No need to elevate".to_owned());
    }
    Some(trigger_elevation().err().map_or(String::new(), |e| e.to_string()))
}

pub fn handle_cm_start_request() {
    if !system_session_server_running() {
        if let Err(e) = trigger_elevation() {
            log::error!("Failed to start system session server from cm: {:?}", e);
        }
    }
}

/// True quando já existe um `--server-session` SYSTEM rodando.
pub fn system_session_server_running() -> bool {
    unsafe {
        let h = OpenMutexW(SYNCHRONIZE, FALSE, session_mutex_name().as_ptr());
        if !h.is_null() {
            CloseHandle(h);
            return true;
        }
    }
    std::io::Error::last_os_error().raw_os_error() == Some(ERROR_ACCESS_DENIED as i32)
}

const USER_SERVER_STOP_OPTION: &str = "stop-service";
static YIELD_WATCHING: AtomicBool = AtomicBool::new(false);

// Override em memória: não persiste e faz o mediator fechar as conexões atuais ao reiniciar.
fn set_user_server_yielded(yielded: bool) {
    {
        let mut overwrite = hbb_common::config::OVERWRITE_SETTINGS.write().unwrap();
        if yielded {
            overwrite.insert(USER_SERVER_STOP_OPTION.to_owned(), "Y".to_owned());
        } else {
            overwrite.remove(USER_SERVER_STOP_OPTION);
        }
    }
    crate::RendezvousMediator::restart();
}

/// Faz o servidor do usuário parar de registrar o ID enquanto o servidor SYSTEM existir.
fn yield_to_system_server() {
    if YIELD_WATCHING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        let mut yielded = false;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let running = system_session_server_running();
            if running && !yielded {
                log::info!("[rech] system session server detected, user server yielding");
                set_user_server_yielded(true);
                yielded = true;
            } else if !running && yielded {
                log::info!("[rech] system session server gone, user server resuming");
                set_user_server_yielded(false);
                break;
            }
        }
        YIELD_WATCHING.store(false, Ordering::SeqCst);
    });
}

/// Aciona a elevação do modo "servidor SYSTEM dono da sessão": grava o handoff de
/// config e dispara a cadeia `--portable-service` -> UAC -> SYSTEM -> `supervise`.
pub fn trigger_elevation() -> hbb_common::ResultType<()> {
    log::info!("[rech] system session server: elevation requested");
    let dir = crate::rech::session_handoff_dir()?;
    crate::rech::export_session_config_for_system(&dir)?;
    register_teardown_shutdown_hook();
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    let arg = format!("--portable-service {}{}", DIR_ARG_PREFIX, hex_encode(&dir.to_string_lossy()));
    match crate::platform::run_background(&exe, &arg) {
        Ok(true) => {
            log::info!("[rech] system session server: elevation chain started");
            yield_to_system_server();
            Ok(())
        }
        Ok(false) => hbb_common::bail!("Failed to run system session server elevation"),
        Err(err) => hbb_common::bail!("Failed to run system session server elevation: {}", err),
    }
}

const SESSION_ARG_PREFIX: &str = "--server-session-id=";

/// Argumento com a sessão do processo atual, repassado ao `--run-as-system`.
pub fn session_arg() -> Option<String> {
    let session_id = crate::platform::windows::get_current_process_session_id()?;
    Some(format!("{}{}", SESSION_ARG_PREFIX, session_id))
}

pub fn session_id_from_args() -> Option<u32> {
    std::env::args().find_map(|arg| arg.strip_prefix(SESSION_ARG_PREFIX)?.parse().ok())
}

/// Supervisor transitório do servidor SYSTEM do modo portable.
///
/// Roda no processo SYSTEM `--run-as-system`, preso à sessão de quem pediu a elevação (`session_id`),
/// com o handoff em `dir`. Quando o servidor sai, limpa o handoff e a config gravada no perfil do SYSTEM.
#[tokio::main(flavor = "current_thread")]
pub async fn supervise(session_id: u32, dir: std::path::PathBuf) {
    log::info!("[rech] system session supervisor started, session {}", session_id);
    let h_process = spawn_session_server(session_id, &dir);
    if h_process.is_null() {
        log::error!("Failed to launch initial system session server");
        cleanup_system_side(&dir);
        return;
    }
    while !server_exited(h_process) {
        hbb_common::sleep(SUPERVISE_INTERVAL_SECS).await;
    }
    log::info!("system session server exited");
    close_handle(h_process);
    cleanup_system_side(&dir);
}

/// Remove o handoff e a config persistida no perfil do SYSTEM.
fn cleanup_system_side(dir: &std::path::Path) {
    crate::rech::wipe_session_config(dir, true);
    let _ = std::fs::remove_file(hbb_common::config::Config::file());
    let _ = std::fs::remove_file(hbb_common::config::Config2::file());
}

/// Saída do app portable: limpa o handoff e pede ao servidor SYSTEM que encerre (arquivo de parada).
pub extern "C" fn teardown() {
    if let Ok(dir) = crate::rech::session_handoff_dir() {
        crate::rech::wipe_session_config(&dir, false);
        crate::rech::request_session_server_stop(&dir);
    }
}

/// Registra a limpeza de saída do app portable (uma única vez por processo).
pub fn register_teardown_shutdown_hook() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        shutdown_hooks::add_shutdown_hook(teardown);
    });
}

/// Watchdog do processo `--server-session`: encerra ao fim da conexão remota.
///
/// Enquanto não houver conexão, aguarda; se nenhuma chegar dentro de
/// `NO_CONNECTION_TIMEOUT`, encerra para não deixar um servidor SYSTEM órfão.
pub fn watch_remote_connections(dir: std::path::PathBuf) {
    let started = Instant::now();
    let mut saw_remote = false;
    let mut dropped_since: Option<Instant> = None;
    loop {
        std::thread::sleep(Duration::from_secs(1));
        if crate::rech::session_server_stop_requested(&dir) {
            log::info!("stop requested by the portable app, exiting system session server");
            exit_server(&dir);
        }
        let remote_count = crate::server::AUTHED_CONNS
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.conn_type == crate::server::AuthConnType::Remote)
            .count();
        if remote_count > 0 {
            saw_remote = true;
            dropped_since = None;
        } else if saw_remote {
            let since = *dropped_since.get_or_insert_with(Instant::now);
            if since.elapsed() > REMOTE_DROP_GRACE {
                log::info!("remote disconnected, exiting system session server");
                exit_server(&dir);
            }
        } else if started.elapsed() > NO_CONNECTION_TIMEOUT {
            log::info!("no remote connection within timeout, exiting system session server");
            exit_server(&dir);
        }
    }
}

/// Remove o handoff e a config persistida no perfil do SYSTEM antes de sair.
fn exit_server(dir: &std::path::Path) -> ! {
    cleanup_system_side(dir);
    std::process::exit(0);
}
