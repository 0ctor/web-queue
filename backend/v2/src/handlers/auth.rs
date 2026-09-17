use crate::models::auth::AuthUser;
use crate::utils::db_error::{is_db_infra_error, service_unavailable_db};
use crate::utils::response::ApiResponse;
use actix_web::{HttpRequest, HttpResponse};
use sqlx::MySqlPool;

#[derive(sqlx::FromRow)]
struct AuthContextRow {
    user_uuid: String,
    profile: String,
    company_uuid: String,
    realm: String,
    fullname: Option<String>,
    email: Option<String>,
}

fn user_roles_for_profile(profile: &str) -> Vec<String> {
    if profile == "owner" {
        vec![
            "admin".to_string(),
            "manager".to_string(),
            "user".to_string(),
        ]
    } else {
        vec!["employee".to_string(), "user".to_string()]
    }
}

fn auth_user_from_row(row: AuthContextRow) -> AuthUser {
    AuthUser {
        user_uuid: row.user_uuid,
        user_profile: row.profile.clone(),
        user_roles: user_roles_for_profile(&row.profile),
        company_uuid: row.company_uuid,
        instance_name: row.realm,
        user_name: row.fullname.unwrap_or_else(|| "Usuário".to_string()),
        user_email: row.email.unwrap_or_default(),
    }
}

pub fn auth_error_response(detail: String) -> HttpResponse {
    if is_db_infra_error(&detail) {
        return service_unavailable_db(detail);
    }
    HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
        "Unauthorized".to_string(),
        detail,
    ))
}

async fn authenticate_token(pool: &MySqlPool, token: &str) -> Result<AuthUser, String> {
    let ctx = sqlx::query_as::<_, AuthContextRow>(
        r#"SELECT user_uuid, profile, company_uuid, realm, fullname, email FROM (
            SELECT
                d.user_uuid AS user_uuid,
                'owner' AS profile,
                c.uuid AS company_uuid,
                c.realm AS realm,
                o.fullname AS fullname,
                o.email AS email,
                1 AS priority
            FROM tbl_users_devices d
            INNER JOIN tbl_company_manager cm ON cm.owner_uuid = d.user_uuid
            INNER JOIN tbl_company c ON c.uuid = cm.company_uuid
            LEFT JOIN tbl_owner o ON o.uuid = cm.owner_uuid
            WHERE d.token_api = ?
            UNION ALL
            SELECT
                d.user_uuid AS user_uuid,
                'employee' AS profile,
                c.uuid AS company_uuid,
                c.realm AS realm,
                e.fullname AS fullname,
                e.email AS email,
                2 AS priority
            FROM tbl_users_devices d
            INNER JOIN tbl_company_employee ce ON ce.employee_uuid = d.user_uuid
            INNER JOIN tbl_company c ON c.uuid = ce.company_uuid
            LEFT JOIN tbl_employee e ON e.uuid = ce.employee_uuid
            WHERE d.token_api = ?
        ) AS ctx
        ORDER BY priority
        LIMIT 1"#,
    )
    .bind(token)
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Invalid token".to_string())?;

    Ok(auth_user_from_row(ctx))
}

pub async fn extract_auth_user(
    req: &HttpRequest,
    pool: &MySqlPool,
) -> Result<AuthUser, HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Unauthorized".to_string(),
                "Authorization header required".to_string(),
            ))
        })?
        .to_str()
        .map_err(|_| {
            HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Unauthorized".to_string(),
                "Invalid Authorization header".to_string(),
            ))
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "Unauthorized".to_string(),
            "Invalid Authorization format. Use: Bearer <token>".to_string(),
        )));
    }

    let token = auth_header.trim_start_matches("Bearer ").trim();
    authenticate_token(pool, token)
        .await
        .map_err(auth_error_response)
}

/// Resolve company: prefer session company; allow X-Company-Uuid only when it matches
/// a company the user belongs to (same token company for v1).
pub fn resolve_company_uuid(auth: &AuthUser, header_company: Option<&str>) -> String {
    if let Some(h) = header_company.map(str::trim).filter(|s| !s.is_empty()) {
        if h == auth.company_uuid {
            return h.to_string();
        }
        // Keep session company as source of truth (anti-spoof).
    }
    auth.company_uuid.clone()
}

pub fn company_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("X-Company-Uuid")
        .or_else(|| req.headers().get("x-company-uuid"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
