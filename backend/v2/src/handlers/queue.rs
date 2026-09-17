use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
use uuid::Uuid;

use crate::domain::{format_display_name, generate_display_code, normalize_display_code};
use crate::handlers::auth::{extract_auth_user, resolve_company_uuid};
use crate::utils::db_error::http_response_for_sqlx_error;
use crate::utils::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct CallRequest {
    pub appointment_uuid: String,
    pub patient_uuid: String,
    pub patient_name: String,
    pub employee_uuid: String,
    pub employee_name: Option<String>,
    pub room_name: Option<String>,
    pub scheduled_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettingsUpdate {
    pub privacy_mode: Option<String>,
    pub sound_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CallData {
    pub appointment_uuid: String,
    pub display_name: String,
    pub room_name: Option<String>,
    pub employee_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SettingsData {
    pub display_code: String,
    pub privacy_mode: String,
    pub sound_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayCall {
    pub display_name: String,
    pub room_name: Option<String>,
    pub employee_name: Option<String>,
    pub called_at: String,
}

#[derive(Debug, Serialize)]
pub struct DisplayData {
    pub current: Option<DisplayCall>,
    pub recent: Vec<DisplayCall>,
    pub sound_enabled: bool,
    pub privacy_mode: String,
}

struct SettingsRow {
    uuid: String,
    display_code: String,
    privacy_mode: String,
    sound_enabled: i8,
}

fn sqlx_or_error(err: sqlx::Error) -> HttpResponse {
    http_response_for_sqlx_error(&err).unwrap_or_else(|| {
        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            "Database error".to_string(),
            err.to_string(),
        ))
    })
}

fn now_utc() -> chrono::NaiveDateTime {
    Utc::now().naive_utc()
}

async fn load_settings(pool: &MySqlPool, company_uuid: &str) -> Result<SettingsRow, sqlx::Error> {
    sqlx::query(
        r#"SELECT uuid, display_code, privacy_mode, sound_enabled
           FROM tbl_queue_settings
           WHERE company_uuid = ? AND deleted_at IS NULL
           LIMIT 1"#,
    )
    .bind(company_uuid)
    .fetch_optional(pool)
    .await?
    .map(|row| SettingsRow {
        uuid: row.get("uuid"),
        display_code: row.get("display_code"),
        privacy_mode: row.get("privacy_mode"),
        sound_enabled: row.get("sound_enabled"),
    })
    .map(Ok)
    .unwrap_or(Err(sqlx::Error::RowNotFound))
}

async fn ensure_settings(pool: &MySqlPool, company_uuid: &str) -> Result<SettingsRow, sqlx::Error> {
    match load_settings(pool, company_uuid).await {
        Ok(existing) => return Ok(existing),
        Err(sqlx::Error::RowNotFound) => {}
        Err(e) => return Err(e),
    }

    for _ in 0..8 {
        let mut bytes = [0u8; 6];
        rand::thread_rng().fill_bytes(&mut bytes);
        let code = generate_display_code(&bytes);
        let uuid = Uuid::new_v4().to_string();
        let insert = sqlx::query(
            r#"INSERT INTO tbl_queue_settings
               (uuid, company_uuid, display_code, privacy_mode, sound_enabled)
               VALUES (?, ?, ?, 'first_last_initial', 1)"#,
        )
        .bind(&uuid)
        .bind(company_uuid)
        .bind(&code)
        .execute(pool)
        .await;
        match insert {
            Ok(_) => {
                return Ok(SettingsRow {
                    uuid,
                    display_code: code,
                    privacy_mode: "first_last_initial".to_string(),
                    sound_enabled: 1,
                });
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Duplicate") || msg.contains("duplicate") {
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(sqlx::Error::Protocol(
        "failed to allocate unique display_code".into(),
    ))
}

async fn upsert_and_call(
    pool: &MySqlPool,
    company_uuid: &str,
    user_uuid: &str,
    body: &CallRequest,
    privacy_mode: &str,
) -> Result<CallData, sqlx::Error> {
    let appointment_uuid = body.appointment_uuid.trim();
    let patient_uuid = body.patient_uuid.trim();
    let patient_name = body.patient_name.trim();
    let employee_uuid = body.employee_uuid.trim();
    if appointment_uuid.is_empty() || patient_uuid.is_empty() || patient_name.is_empty() {
        return Err(sqlx::Error::Protocol("missing required fields".into()));
    }

    let now = now_utc();
    let ticket_uuid = Uuid::new_v4().to_string();
    let existing = sqlx::query(
        r#"SELECT uuid FROM tbl_queue_tickets
           WHERE company_uuid = ? AND appointment_uuid = ? AND deleted_at IS NULL
           LIMIT 1"#,
    )
    .bind(company_uuid)
    .bind(appointment_uuid)
    .fetch_optional(pool)
    .await?;

    let ticket_uuid = if let Some(row) = existing {
        let uuid: String = row.get("uuid");
        sqlx::query(
            r#"UPDATE tbl_queue_tickets
               SET status = 'called',
                   called_at = ?,
                   patient_name = ?,
                   employee_uuid = ?,
                   employee_name = ?,
                   room_name = ?,
                   updated_at = ?
               WHERE uuid = ? AND company_uuid = ? AND deleted_at IS NULL"#,
        )
        .bind(now)
        .bind(patient_name)
        .bind(employee_uuid)
        .bind(body.employee_name.as_deref())
        .bind(body.room_name.as_deref())
        .bind(now)
        .bind(&uuid)
        .bind(company_uuid)
        .execute(pool)
        .await?;
        uuid
    } else {
        sqlx::query(
            r#"INSERT INTO tbl_queue_tickets
               (uuid, company_uuid, appointment_uuid, patient_uuid, patient_name,
                employee_uuid, employee_name, room_name, scheduled_at, status,
                arrived_at, called_at, created_by)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, 'called', ?, ?, ?)"#,
        )
        .bind(&ticket_uuid)
        .bind(company_uuid)
        .bind(appointment_uuid)
        .bind(patient_uuid)
        .bind(patient_name)
        .bind(employee_uuid)
        .bind(body.employee_name.as_deref())
        .bind(body.room_name.as_deref())
        .bind(now)
        .bind(now)
        .bind(user_uuid)
        .execute(pool)
        .await?;
        ticket_uuid
    };

    let display_name = format_display_name(patient_name, privacy_mode);
    let call_uuid = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO tbl_queue_calls
           (uuid, company_uuid, ticket_uuid, appointment_uuid, display_name,
            room_name, employee_name, called_by, called_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&call_uuid)
    .bind(company_uuid)
    .bind(&ticket_uuid)
    .bind(appointment_uuid)
    .bind(&display_name)
    .bind(body.room_name.as_deref())
    .bind(body.employee_name.as_deref())
    .bind(user_uuid)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(CallData {
        appointment_uuid: appointment_uuid.to_string(),
        display_name,
        room_name: body.room_name.clone(),
        employee_name: body.employee_name.clone(),
    })
}

#[post("/v2/queue/call")]
pub async fn call_patient(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<CallRequest>,
) -> impl Responder {
    let auth = match extract_auth_user(&req, pool.get_ref()).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    let company_uuid = resolve_company_uuid(&auth, None);
    let settings = match ensure_settings(pool.get_ref(), &company_uuid).await {
        Ok(s) => s,
        Err(e) => return sqlx_or_error(e),
    };
    match upsert_and_call(
        pool.get_ref(),
        &company_uuid,
        &auth.user_uuid,
        &body,
        &settings.privacy_mode,
    )
    .await
    {
        Ok(data) => HttpResponse::Ok().json(ApiResponse::success(
            "Paciente chamado".to_string(),
            data,
        )),
        Err(sqlx::Error::Protocol(msg)) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Dados inválidos".to_string(),
            msg.to_string(),
        )),
        Err(e) => sqlx_or_error(e),
    }
}

#[get("/v2/queue/settings")]
pub async fn get_settings(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    let auth = match extract_auth_user(&req, pool.get_ref()).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    let company_uuid = resolve_company_uuid(&auth, None);
    match ensure_settings(pool.get_ref(), &company_uuid).await {
        Ok(s) => HttpResponse::Ok().json(ApiResponse::success(
            "ok".to_string(),
            SettingsData {
                display_code: s.display_code,
                privacy_mode: s.privacy_mode,
                sound_enabled: s.sound_enabled != 0,
            },
        )),
        Err(e) => sqlx_or_error(e),
    }
}

#[put("/v2/queue/settings")]
pub async fn update_settings(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<SettingsUpdate>,
) -> impl Responder {
    let auth = match extract_auth_user(&req, pool.get_ref()).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    let company_uuid = resolve_company_uuid(&auth, None);
    let settings = match ensure_settings(pool.get_ref(), &company_uuid).await {
        Ok(s) => s,
        Err(e) => return sqlx_or_error(e),
    };
    let privacy = body
        .privacy_mode
        .as_deref()
        .unwrap_or(&settings.privacy_mode);
    if !matches!(privacy, "full" | "first_last_initial" | "initials") {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Dados inválidos".to_string(),
            "privacy_mode inválido".to_string(),
        ));
    }
    let sound = body
        .sound_enabled
        .map(|v| if v { 1 } else { 0 })
        .unwrap_or(settings.sound_enabled);
    match sqlx::query(
        r#"UPDATE tbl_queue_settings
           SET privacy_mode = ?, sound_enabled = ?
           WHERE uuid = ? AND company_uuid = ? AND deleted_at IS NULL"#,
    )
    .bind(privacy)
    .bind(sound)
    .bind(&settings.uuid)
    .bind(&company_uuid)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success(
            "ok".to_string(),
            SettingsData {
                display_code: settings.display_code,
                privacy_mode: privacy.to_string(),
                sound_enabled: sound != 0,
            },
        )),
        Err(e) => sqlx_or_error(e),
    }
}

#[get("/v2/queue/display/{code}")]
pub async fn get_display(
    pool: web::Data<MySqlPool>,
    code: web::Path<String>,
) -> impl Responder {
    let code = normalize_display_code(&code);
    if code.len() != 6 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Código inválido".to_string(),
            "use 6 caracteres".to_string(),
        ));
    }
    let settings = match sqlx::query(
        r#"SELECT company_uuid, privacy_mode, sound_enabled
           FROM tbl_queue_settings
           WHERE display_code = ? AND deleted_at IS NULL
           LIMIT 1"#,
    )
    .bind(&code)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Painel não encontrado".to_string(),
                "código desconhecido".to_string(),
            ));
        }
        Err(e) => return sqlx_or_error(e),
    };
    let company_uuid: String = settings.get("company_uuid");
    let privacy_mode: String = settings.get("privacy_mode");
    let sound_enabled: i8 = settings.get("sound_enabled");

    let rows = match sqlx::query(
        r#"SELECT display_name, room_name, employee_name, called_at
           FROM tbl_queue_calls
           WHERE company_uuid = ? AND deleted_at IS NULL
           ORDER BY called_at DESC
           LIMIT 6"#,
    )
    .bind(&company_uuid)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => return sqlx_or_error(e),
    };

    let recent: Vec<DisplayCall> = rows
        .into_iter()
        .map(|row| {
            let called_at: chrono::NaiveDateTime = row.get("called_at");
            DisplayCall {
                display_name: row.get("display_name"),
                room_name: row.get("room_name"),
                employee_name: row.get("employee_name"),
                called_at: called_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            }
        })
        .collect();
    let current = recent.first().cloned();

    HttpResponse::Ok().json(ApiResponse::success(
        "ok".to_string(),
        DisplayData {
            current,
            recent: recent.into_iter().skip(1).collect(),
            sound_enabled: sound_enabled != 0,
            privacy_mode,
        },
    ))
}
