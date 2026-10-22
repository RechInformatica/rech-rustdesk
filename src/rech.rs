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
}
