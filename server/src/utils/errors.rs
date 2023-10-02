use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum PunchTaskError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "the task is already in progress")]
    TaskAlreadyInProgress,

    #[display(fmt = "no task with the given name found")]
    TaskNotFound,

    #[display(fmt = "no such task in progress found")]
    InProgressTaskNotFound,
}

impl error::ResponseError for PunchTaskError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            PunchTaskError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            PunchTaskError::TaskAlreadyInProgress => StatusCode::BAD_REQUEST,
            PunchTaskError::InProgressTaskNotFound => StatusCode::BAD_REQUEST,
            PunchTaskError::TaskNotFound => StatusCode::NOT_FOUND,
        }
    }
}
