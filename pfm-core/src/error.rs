use thiserror::Error;

/// Wrapper around anyhow::Error
pub trait BaseError: Sized + std::error::Error + Send + Sync + 'static {
    fn new(err: impl Into<anyhow::Error>) -> Self;

    fn from_msg<S: AsRef<str>>(message: S) -> Self {
        let err = anyhow::anyhow!("{}", message.as_ref());
        Self::new(err)
    }

    fn from_err<E>(error: E) -> Self
    where
        E: Into<anyhow::Error> + std::error::Error + Send + Sync + 'static,
    {
        Self::new(error)
    }

    fn with_context<S: AsRef<str>>(
        message: S,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        let err = anyhow::Error::new(source).context(message.as_ref().to_string());
        Self::new(err)
    }

    fn cause(&self) -> String {
        self.source()
            .map(|err| err.to_string())
            .unwrap_or("Error undefined".to_string())
    }

    fn detail(&self) -> String {
        let error = self.to_string();
        let cause = BaseError::cause(self);
        format!("{} \nCaused by: {}", error, cause)
    }
}

#[derive(Debug, Error)]
#[error("Error: {0}")]
pub struct Error(#[from] anyhow::Error);

impl BaseError for Error {
    fn new(err: impl Into<anyhow::Error>) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, Error)]
#[error("Client error: {0}")]
pub struct ClientError(#[from] anyhow::Error);

impl BaseError for ClientError {
    fn new(err: impl Into<anyhow::Error>) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, Error)]
#[error("Internal error: {0}")]
pub struct InternalError(#[from] anyhow::Error);

impl BaseError for InternalError {
    fn new(err: impl Into<anyhow::Error>) -> Self {
        Self(err.into())
    }
}

pub trait AsError<T> {
    fn as_err(self) -> Result<T, Error>;
}

pub trait AsClientError<T> {
    fn as_client_err(self) -> Result<T, ClientError>;
}

pub trait AsInternalError<T> {
    fn as_internal_err(self) -> Result<T, InternalError>;
}

impl<T, E> AsError<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn as_err(self) -> Result<T, Error> {
        self.map_err(|e| Error(e.into()))
    }
}

impl<T, E> AsClientError<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn as_client_err(self) -> Result<T, ClientError> {
        self.map_err(|e| ClientError(e.into()))
    }
}

impl<T, E> AsInternalError<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn as_internal_err(self) -> Result<T, InternalError> {
        self.map_err(|e| InternalError(e.into()))
    }
}
