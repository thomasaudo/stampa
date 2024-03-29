use std::fmt::{self, Display};

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug)]
pub enum AppErrorType {
    DbError,
    NotFoundError,
    NotInProject,
    S3Error,
    LoginError,
    AvatarGenerationError,
    UserExistError,
    UnvalidFormError,
}

#[derive(Debug)]
pub struct AppError {
    pub message: Option<String>,
    pub cause: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    fn message(&self) -> String {
        match &*self {
            AppError {
                message: Some(message),
                cause: _,
                error_type: _,
            } => message.clone(),
            AppError {
                message: None,
                cause: _,
                error_type: _,
            } => "An internal error occured".to_string(),
        }
    }

    pub fn db_error(error: impl ToString) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: crate::errors::AppErrorType::DbError,
        }
    }

    pub fn avatat_generation_error(error: impl ToString) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: crate::errors::AppErrorType::DbError,
        }
    }

    pub fn not_found_error(ressource_id: impl ToString) -> AppError {
        AppError {
            message: Some(format!(
                "The ressource {} was not found",
                ressource_id.to_string()
            )),
            cause: Some(ressource_id.to_string()),
            error_type: crate::errors::AppErrorType::AvatarGenerationError,
        }
    }

    pub fn not_in_project_error(ressource_id: impl ToString) -> AppError {
        AppError {
            message: Some(format!(
                "The user {} is not in the project",
                ressource_id.to_string()
            )),
            cause: Some(ressource_id.to_string()),
            error_type: crate::errors::AppErrorType::NotInProject,
        }
    }

    pub fn s3_error(error: impl ToString) -> AppError {
        AppError {
            message: Some(format!("Internal error")),
            cause: Some(error.to_string()),
            error_type: crate::errors::AppErrorType::NotInProject,
        }
    }

    pub fn fs_error(error: impl ToString) -> AppError {
        AppError {
            message: Some(format!("Internal error")),
            cause: Some(error.to_string()),
            error_type: crate::errors::AppErrorType::NotInProject,
        }
    }

    pub fn login_error(username: impl ToString) -> AppError {
        AppError {
            message: Some(format!("Can not login user {}.", username.to_string())),
            cause: Some(format!("Can not login user {}.", username.to_string())),
            error_type: crate::errors::AppErrorType::LoginError,
        }
    }

    pub fn user_exist_error(username: impl ToString) -> AppError {
        AppError {
            message: Some(format!("User {} already exist.", username.to_string())),
            cause: Some(format!("User {} already exist", username.to_string())),
            error_type: crate::errors::AppErrorType::UserExistError,
        }
    }

    pub fn unvalid_form_error(error: impl ToString) -> AppError {
        AppError {
            message: Some(error.to_string()),
            cause: Some(error.to_string()),
            error_type: crate::errors::AppErrorType::UnvalidFormError,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::DbError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::NotFoundError => StatusCode::NOT_FOUND,
            AppErrorType::NotInProject => StatusCode::UNAUTHORIZED,
            AppErrorType::LoginError => StatusCode::UNAUTHORIZED,
            AppErrorType::S3Error => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::AvatarGenerationError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::UserExistError => StatusCode::UNAUTHORIZED,
            AppErrorType::UnvalidFormError => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}
