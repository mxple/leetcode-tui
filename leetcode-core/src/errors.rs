use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LcAppError {
    #[error("Looks like your cookies has been expired kindly update your cookies in config.toml")]
    CookiesExpiredError,

    #[error("Deserialization/serialization failed: {0}")]
    DeserializeError(#[from] serde_json::Error),

    #[error("Network request error.")]
    RequestError(#[from] reqwest::Error),

    #[error("Status {code:?}: {contents:?}")]
    StatusCodeError { code: String, contents: String },

    #[error("Error while building reqwest client: {0}")]
    ClientBuildError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Language does not exist for question {0}")]
    LanguageDoesNotExistError(String),

    #[error("Filename format does not match: {0}")]
    FileNameFormatDoesNotMatch(String),

    #[error("Couldn't parse language id: {0}")]
    LangIdParseError(#[from] ParseIntError),
}

pub type AppResult<T> = Result<T, LcAppError>;
