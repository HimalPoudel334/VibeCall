use actix_web::error;

use crate::shared::response::AppError;

pub fn json_error_handler(
    err: error::JsonPayloadError,
    _req: &actix_web::HttpRequest,
) -> error::Error {
    let cleaned_message = err
        .to_string()
        .replace("Json deserialize error: ", "")
        .replace("invalid type: string \"", "Invalid value: ")
        .to_string();

    AppError::Validation(cleaned_message).into()
}
