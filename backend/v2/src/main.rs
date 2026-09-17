use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::env;
use std::net::SocketAddr;

use web_queue_api::config::{
    backend_port_from_env, mysql_acquire_timeout_secs_from_env,
    mysql_pool_options_from_env_values, mysql_startup_connect_attempts_from_env,
    request_max_in_flight_from_env,
};
use web_queue_api::server::{configure_api_services, is_wildcard_cors, parse_cors_origins};
use web_queue_api::utils::request_concurrency::RequestConcurrencyLimiter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if dotenvy::dotenv().is_err() {
        let _ = dotenvy::from_path("../.env");
        let _ = dotenvy::from_path("../../.env");
    }
    env_logger::init();

    let db_user = env::var("DB_USER").expect("DB_USER must be set");
    let db_pass = env::var("DB_PASS").expect("DB_PASS must be set");
    let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".into());
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");

    let pool_opts = mysql_pool_options_from_env_values(
        env::var("DB_TEST_BEFORE_ACQUIRE").ok().as_deref(),
        env::var("DB_MAX_CONNECTIONS").ok().as_deref(),
    );
    let acquire_secs =
        mysql_acquire_timeout_secs_from_env(env::var("DB_ACQUIRE_TIMEOUT_SEC").ok().as_deref());
    let connect_attempts = mysql_startup_connect_attempts_from_env(
        env::var("DB_CONNECT_ATTEMPTS").ok().as_deref(),
    );

    let database_url = format!("mysql://{db_user}:{db_pass}@{db_host}:{db_port}/{db_name}");

    use sqlx::mysql::MySqlPoolOptions;
    let mut pool = None;
    for attempt in 1..=connect_attempts {
        let options = MySqlPoolOptions::new()
            .max_lifetime(std::time::Duration::from_secs(1800))
            .idle_timeout(std::time::Duration::from_secs(600))
            .acquire_timeout(std::time::Duration::from_secs(acquire_secs))
            .test_before_acquire(pool_opts.test_before_acquire)
            .max_connections(pool_opts.max_connections);
        match options.connect(&database_url).await {
            Ok(p) => {
                pool = Some(p);
                break;
            }
            Err(e) => {
                eprintln!("MySQL connect attempt {attempt}/{connect_attempts} failed: {e}");
                if attempt < connect_attempts {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
    let pool = pool.expect("Failed to connect to MySQL");

    let port = backend_port_from_env(env::var("BACKEND_PORT").ok().as_deref());
    let max_in_flight =
        request_max_in_flight_from_env(env::var("REQUEST_MAX_IN_FLIGHT").ok().as_deref());
    let cors_origins = parse_cors_origins(
        env::var("CORS_ORIGINS")
            .ok()
            .or_else(|| env::var("API_CORS_ORIGIN").ok())
            .as_deref(),
    );
    let wildcard = is_wildcard_cors(&cors_origins);
    let pool_data = web::Data::new(pool);

    let addr: SocketAddr = format!("0.0.0.0:{port}").parse().unwrap();
    println!("web-queue-api listening on {port}");

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);
        if wildcard {
            cors = cors.allow_any_origin();
        } else {
            for o in &cors_origins {
                cors = cors.allowed_origin(o);
            }
            cors = cors.supports_credentials();
        }

        App::new()
            .app_data(pool_data.clone())
            .wrap(RequestConcurrencyLimiter::new(max_in_flight))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)
            .configure(configure_api_services)
    })
    .bind(addr)?
    .run()
    .await
}
