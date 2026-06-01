use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ZooEmbeddingError {
    #[error("Request failed")]
    RequestFailed(String),
    #[error("Invalid model architecture")]
    InvalidModelArchitecture,
    #[error("Unimplemented model dimensions")]
    UnimplementedModelDimensions(String),
    #[error("Failed embedding generation")]
    FailedEmbeddingGeneration(String),
}

impl From<reqwest::Error> for ZooEmbeddingError {
    fn from(error: reqwest::Error) -> Self {
        ZooEmbeddingError::RequestFailed(error.to_string())
    }
}
