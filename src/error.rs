use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Io Error: {0}")]
  IoError(#[from] std::io::Error),
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let status = match &self {
      Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, self.to_string()).into_response()
  }
}
