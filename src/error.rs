use std::{borrow::Cow, collections::HashMap};

use axum::{
    body::{Bytes, Full},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 401 Unauthorized
    #[error("authentication required")]
    Unauthorized,

    /// 403 Forbidden
    #[error("user may not perform that action")]
    Forbidden,

    /// 404 Not Found
    #[error("request path not found")]
    NotFound,

    /// 422 Unprocessable Entity
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// 500 Internal Server Error from anyhow::Error
    #[error("an unexpected error occured: {0}")]
    Unexpected(#[from] anyhow::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();
        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if let Self::UnprocessableEntity { errors } = self {
            let body = serde_json::json!({ "errors": errors });
            (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
        } else {
            (self.status_code(), Full::new(Bytes::from(self.to_string()))).into_response()
        }
    }
}
