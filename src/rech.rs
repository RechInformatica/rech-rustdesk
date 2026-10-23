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
    hbb_common::config::BUILTIN_SETTINGS
        .write()
        .unwrap()
        .insert("hide-server-settings".to_owned(), "Y".to_owned());
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
        format!("Rech Rustdesk {} (RustDesk {})", rech, base)
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
