use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::zoo_message::zoo_message::ZooMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetryMessage {
    pub retry_count: u32,
    pub message: ZooMessage,
    pub save_to_db_flag: bool,
    pub peer: (SocketAddr, String),
}
