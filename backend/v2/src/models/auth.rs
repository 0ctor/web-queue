use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub user_uuid: String,
    pub user_profile: String,
    pub user_roles: Vec<String>,
    pub company_uuid: String,
    pub instance_name: String,
    pub user_name: String,
    pub user_email: String,
}
