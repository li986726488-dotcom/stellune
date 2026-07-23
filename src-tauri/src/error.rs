use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            code: "VALIDATION_ERROR".into(),
            message: message.into(),
            retryable: false,
        }
    }

    pub fn profile_incomplete() -> Self {
        Self {
            code: "PROFILE_INCOMPLETE".into(),
            message: "请先完成生日设置。".into(),
            retryable: false,
        }
    }

    pub fn database(error: impl std::fmt::Display) -> Self {
        Self {
            code: "DATABASE_ERROR".into(),
            message: format!("本地数据暂时无法读取：{error}"),
            retryable: true,
        }
    }

    pub fn astrology(code: &str, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }

    pub fn internal(error: impl std::fmt::Display) -> Self {
        Self {
            code: "INTERNAL_ERROR".into(),
            message: format!("应用暂时无法完成这次操作：{error}"),
            retryable: true,
        }
    }
}
