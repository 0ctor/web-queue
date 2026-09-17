use actix_web::{
    get,
    http::{header, StatusCode},
    web, HttpResponse, Responder,
};
use sqlx::MySqlPool;
use std::time::Duration;

#[get("/health/live")]
pub async fn health_live() -> impl Responder {
    standard_health_response(StatusCode::OK, "ok")
}

#[get("/health/ready")]
pub async fn health_ready_public(pool: web::Data<MySqlPool>) -> impl Responder {
    match probe_mysql(pool.get_ref()).await {
        Ok(()) => standard_health_response(StatusCode::OK, "ok"),
        Err(_) => standard_health_response(StatusCode::SERVICE_UNAVAILABLE, "not_ready"),
    }
}

#[get("/v2/health")]
pub async fn health_legacy(pool: web::Data<MySqlPool>) -> impl Responder {
    match probe_mysql(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "version": "2.0.0",
            "api": "web-queue-api"
        })),
        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "version": "2.0.0",
            "api": "web-queue-api"
        })),
    }
}

#[get("/v2/health/ready")]
pub async fn health_ready_v2(pool: web::Data<MySqlPool>) -> impl Responder {
    match probe_mysql(pool.get_ref()).await {
        Ok(()) => standard_health_response(StatusCode::OK, "ok"),
        Err(_) => standard_health_response(StatusCode::SERVICE_UNAVAILABLE, "not_ready"),
    }
}

async fn probe_mysql(pool: &MySqlPool) -> Result<(), String> {
    let timeout = Duration::from_secs(5);
    match tokio::time::timeout(timeout, sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool))
        .await
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("health probe timeout".to_string()),
    }
}

fn standard_health_response(http_status: StatusCode, status: &str) -> HttpResponse {
    HttpResponse::build(http_status)
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .json(serde_json::json!({
            "status": status,
            "service": "web-queue-api"
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_contract() {
        let response = standard_health_response(StatusCode::OK, "ok");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
    }
}
