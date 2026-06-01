use zoo_message_primitives::{
    schemas::{inbox_name::InboxNameError, zoo_name::ZooNameError},
    zoo_message::zoo_message_error::ZooMessageError,
};
use zoo_sqlite::errors::SqliteManagerError;
use zoo_tools_primitives::tools::error::ToolError;

use crate::llm_provider::error::LLMProviderError;

#[derive(Debug)]
pub struct NodeError {
    pub message: String,
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NodeError {}

impl From<Box<dyn std::error::Error + Send + Sync>> for NodeError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> NodeError {
        NodeError {
            message: format!("{}", err),
        }
    }
}

impl From<Box<dyn std::error::Error + Send>> for NodeError {
    fn from(err: Box<dyn std::error::Error + Send>) -> NodeError {
        NodeError {
            message: format!("{}", err),
        }
    }
}

impl From<std::io::Error> for NodeError {
    fn from(err: std::io::Error) -> NodeError {
        NodeError {
            message: format!("{}", err),
        }
    }
}

impl From<ZooMessageError> for NodeError {
    fn from(err: ZooMessageError) -> NodeError {
        NodeError {
            message: format!("{}", err),
        }
    }
}

impl From<LLMProviderError> for NodeError {
    fn from(error: LLMProviderError) -> Self {
        NodeError {
            message: format!("LLMProviderError occurred: {}", error),
        }
    }
}

impl From<SqliteManagerError> for NodeError {
    fn from(error: SqliteManagerError) -> Self {
        NodeError {
            message: format!("Database error: {}", error),
        }
    }
}

impl From<ToolError> for NodeError {
    fn from(error: ToolError) -> Self {
        NodeError {
            message: format!("{}", error),
        }
    }
}

impl From<InboxNameError> for NodeError {
    fn from(err: InboxNameError) -> NodeError {
        NodeError {
            message: format!("InboxNameError: {}", err),
        }
    }
}

impl From<ZooNameError> for NodeError {
    fn from(error: ZooNameError) -> Self {
        NodeError {
            message: format!("ZooNameError: {}", error),
        }
    }
}

impl From<String> for NodeError {
    fn from(error: String) -> Self {
        NodeError { message: error }
    }
}

impl From<serde_json::Error> for NodeError {
    fn from(err: serde_json::Error) -> NodeError {
        NodeError {
            message: format!("Serde JSON error: {}", err),
        }
    }
}
