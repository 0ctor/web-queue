use actix_web::HttpResponse;

use crate::utils::response::ApiResponse;

pub fn is_db_infra_error(detail: &str) -> bool {
    let d = detail.to_ascii_lowercase();
    d.contains("pool timed out")
        || d.contains("pooltimedout")
        || d.contains("database error:")
        || d.contains("error communicating with the database")
        || d.contains("connection refused")
        || d.contains("broken pipe")
        || d.contains("server has gone away")
        || d.contains("lost connection")
        || d.contains("too many connections")
        || d.contains("can't connect")
        || d.contains("cannot connect")
        || d.contains("connect timed out")
        || d.contains("connection timed out")
}

pub fn service_unavailable_db(detail: String) -> HttpResponse {
    HttpResponse::ServiceUnavailable().json(ApiResponse::<()>::error(
        "Service temporarily unavailable".to_string(),
        detail,
    ))
}

pub fn http_response_for_sqlx_error(err: &sqlx::Error) -> Option<HttpResponse> {
    let detail = format!("Database error: {}", err);
    if is_db_infra_error(&detail) || matches!(err, sqlx::Error::PoolTimedOut) {
        Some(service_unavailable_db(detail))
    } else {
        let s = err.to_string();
        if is_db_infra_error(&s) {
            Some(service_unavailable_db(format!("Database error: {}", s)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_infra_errors() {
        assert!(is_db_infra_error("pool timed out"));
        assert!(!is_db_infra_error("Invalid token"));
    }
}
