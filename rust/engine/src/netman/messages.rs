use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByServer {}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByClient {}
