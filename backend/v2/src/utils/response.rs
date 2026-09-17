use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(message: String, data: T) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String, error_detail: String) -> Self {
        Self {
            success: false,
            message,
            data: None::<T>,
            error: Some(error_detail),
        }
    }
}
