use thiserror::Error;

/// Exit codes per spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    AuthFailure = 2,
    AuthzFailure = 3,
    NotFound = 4,
    RateLimited = 5,
    NetworkError = 6,
    Conflict = 7,
    UserCancelled = 8,
    Sigint = 130,
}

impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Error, Debug)]
pub enum MogError {
    #[error("General error: {0}")]
    General(String),

    #[error("Authentication failure: {0}")]
    Auth(String),

    #[error("Authorization failure (403): {0}")]
    Authz(String),

    #[error("Resource not found (404): {0}")]
    NotFound(String),

    #[error("Rate limited (429): {0}")]
    RateLimited(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Conflict (409): {0}")]
    Conflict(String),

    #[error("User cancelled")]
    UserCancelled,

    #[error("Non-interactive mode: {0}")]
    NonInteractive(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl MogError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            MogError::General(_) => ExitCode::GeneralError,
            MogError::Auth(_) => ExitCode::AuthFailure,
            MogError::Authz(_) => ExitCode::AuthzFailure,
            MogError::NotFound(_) => ExitCode::NotFound,
            MogError::RateLimited(_) => ExitCode::RateLimited,
            MogError::Network(_) => ExitCode::NetworkError,
            MogError::Conflict(_) => ExitCode::Conflict,
            MogError::UserCancelled => ExitCode::UserCancelled,
            MogError::NonInteractive(_) => ExitCode::GeneralError,
            MogError::Config(_) => ExitCode::GeneralError,
            MogError::Validation(_) => ExitCode::GeneralError,
            MogError::Io(_) => ExitCode::GeneralError,
            MogError::SerdeJson(_) => ExitCode::GeneralError,
            MogError::Other(_) => ExitCode::GeneralError,
        }
    }
}

impl From<MogError> for i32 {
    fn from(e: MogError) -> i32 {
        e.exit_code().code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(ExitCode::Success.code(), 0);
        assert_eq!(ExitCode::GeneralError.code(), 1);
        assert_eq!(ExitCode::AuthFailure.code(), 2);
        assert_eq!(ExitCode::AuthzFailure.code(), 3);
        assert_eq!(ExitCode::NotFound.code(), 4);
        assert_eq!(ExitCode::RateLimited.code(), 5);
        assert_eq!(ExitCode::NetworkError.code(), 6);
        assert_eq!(ExitCode::Conflict.code(), 7);
        assert_eq!(ExitCode::UserCancelled.code(), 8);
        assert_eq!(ExitCode::Sigint.code(), 130);
    }

    #[test]
    fn test_error_exit_code_mapping() {
        assert_eq!(MogError::Auth("test".into()).exit_code(), ExitCode::AuthFailure);
        assert_eq!(MogError::Authz("test".into()).exit_code(), ExitCode::AuthzFailure);
        assert_eq!(MogError::NotFound("test".into()).exit_code(), ExitCode::NotFound);
        assert_eq!(MogError::RateLimited("test".into()).exit_code(), ExitCode::RateLimited);
        assert_eq!(MogError::Network("test".into()).exit_code(), ExitCode::NetworkError);
        assert_eq!(MogError::Conflict("test".into()).exit_code(), ExitCode::Conflict);
        assert_eq!(MogError::UserCancelled.exit_code(), ExitCode::UserCancelled);
    }

    #[test]
    fn test_error_to_i32() {
        let code: i32 = MogError::Auth("test".into()).into();
        assert_eq!(code, 2);
        let code: i32 = MogError::NotFound("test".into()).into();
        assert_eq!(code, 4);
    }

    #[test]
    fn test_fallback_exit_codes() {
        assert_eq!(MogError::NonInteractive("x".into()).exit_code(), ExitCode::GeneralError);
        assert_eq!(MogError::Config("x".into()).exit_code(), ExitCode::GeneralError);
        assert_eq!(MogError::Validation("x".into()).exit_code(), ExitCode::GeneralError);
    }

    #[test]
    fn test_error_display() {
        let e = MogError::Auth("bad token".into());
        assert!(e.to_string().contains("bad token"));
    }
}
