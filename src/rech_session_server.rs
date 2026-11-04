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

fn session_mutex_name() -> Vec<u16> {
    format!("Global\\{}_system_session", crate::get_app_name())
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

/// Pipe próprio do servidor SYSTEM; o ACL padrão de um pipe criado por SYSTEM bloqueia escrita do usuário.
pub const SYSTEM_IPC_POSTFIX: &str = "_system_session";

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

fn spawn_session_server(session_id: u32) -> HANDLE {
    log::info!("launching system session server in session {}", session_id);
    match crate::platform::windows::spawn_session_server_as_system(session_id) {
        Ok(h) => h,
        Err(err) => {
            log::error!("Failed to launch system session server: {}", err);
            NULL
        }
    }
}

async fn close_session_server_inner() {
    if let Ok(mut conn) = crate::ipc::connect(1000, SYSTEM_IPC_POSTFIX).await {
        let _ = conn.send(&crate::ipc::Data::Close).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn close_session_server() {
    close_session_server_inner().await;
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
const SYSTEM_START_TIMEOUT: Duration = Duration::from_secs(120);
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
        let started = Instant::now();
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
            } else if !running && started.elapsed() > SYSTEM_START_TIMEOUT {
                log::warn!("[rech] system session server did not start in time");
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
    crate::rech::export_session_config_for_system()?;
    register_teardown_shutdown_hook();
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    match crate::platform::run_background(&exe, "--portable-service") {
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
/// Roda no processo SYSTEM `--run-as-system`, preso à sessão de quem pediu a elevação,
/// e encerra tudo quando o servidor sai (desconexão ou saída do portable).
#[tokio::main(flavor = "current_thread")]
pub async fn supervise(session_id: u32) {
    log::info!("[rech] system session supervisor started, session {}", session_id);
    let h_process = spawn_session_server(session_id);
    if h_process.is_null() {
        log::error!("Failed to launch initial system session server");
        teardown();
        return;
    }
    while !server_exited(h_process) {
        hbb_common::sleep(SUPERVISE_INTERVAL_SECS).await;
    }
    log::info!("system session server exited");
    close_handle(h_process);
    teardown();
}

/// Encerra o servidor SYSTEM e limpa a config temporária do handoff.
pub extern "C" fn teardown() {
    crate::rech::wipe_session_config();
    std::thread::spawn(close_session_server);
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
pub fn watch_remote_connections() {
    let started = Instant::now();
    let mut saw_remote = false;
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let remote_count = crate::server::AUTHED_CONNS
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.conn_type == crate::server::AuthConnType::Remote)
            .count();
        if remote_count > 0 {
            saw_remote = true;
        } else if saw_remote {
            log::info!("remote disconnected, exiting system session server");
            std::process::exit(0);
        } else if started.elapsed() > NO_CONNECTION_TIMEOUT {
            log::info!("no remote connection within timeout, exiting system session server");
            std::process::exit(0);
        }
    }
}
