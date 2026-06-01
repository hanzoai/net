use std::fmt;

#[derive(Debug)]
pub enum ZooMessageError {
    SigningError(String),
    DecryptionError(String),
    EncryptionError(String),
    InvalidMessageSchemaType(String),
    MissingMessageBody(String),
    DeserializationError(String),
    SerializationError(String),
    AlreadyEncrypted(String),
}

impl fmt::Display for ZooMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZooMessageError::SigningError(msg) => write!(f, "SigningError: {}", msg),
            ZooMessageError::DecryptionError(msg) => write!(f, "DecryptionError: {}", msg),
            ZooMessageError::EncryptionError(msg) => write!(f, "EncryptionError: {}", msg),
            ZooMessageError::InvalidMessageSchemaType(msg) => write!(f, "InvalidMessageSchemaType: {}", msg),
            ZooMessageError::MissingMessageBody(msg) => write!(f, "MissingMessageBody: {}", msg),
            ZooMessageError::DeserializationError(msg) => write!(f, "DeserializationError: {}", msg),
            ZooMessageError::SerializationError(msg) => write!(f, "SerializationError: {}", msg),
            ZooMessageError::AlreadyEncrypted(msg) => write!(f, "AlreadyEncrypted: {}", msg),
        }
    }
}

impl std::error::Error for ZooMessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Note: Update this if we wrap other error and we want to return the source (underlying cause).
        None
    }
}
