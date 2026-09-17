use actix_web::web;

use crate::handlers::{health, queue};

pub const DEFAULT_CORS_ORIGINS: &[&str] = &[
    "https://chamadas.octor.com.br",
    "https://fila.octor.com.br",
    "https://mr.octor.com.br",
    "https://prontuario.octor.com.br",
    "https://agenda.octor.com.br",
    "https://auth.octor.com.br",
    "http://10.8.0.9:15183",
    "http://10.8.0.9:15182",
    "http://10.8.0.9:15177",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
    "http://localhost:5178",
    "http://127.0.0.1:5178",
];

pub fn parse_cors_origins(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(s) if !s.trim().is_empty() => s
            .split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(str::to_string)
            .collect(),
        _ => DEFAULT_CORS_ORIGINS
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    }
}

pub fn is_wildcard_cors(origins: &[String]) -> bool {
    origins.iter().any(|o| o == "*")
}

pub fn configure_api_services(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_live)
        .service(health::health_ready_public)
        .service(health::health_legacy)
        .service(health::health_ready_v2)
        .service(queue::call_patient)
        .service(queue::get_settings)
        .service(queue::update_settings)
        .service(queue::get_display);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cors_includes_dev_vpn_apps() {
        let origins = parse_cors_origins(None);
        assert!(origins.iter().any(|o| o == "http://10.8.0.9:15183"));
        assert!(origins.iter().any(|o| o == "http://10.8.0.9:15182"));
        assert!(origins.iter().any(|o| o == "http://10.8.0.9:15177"));
    }
}
