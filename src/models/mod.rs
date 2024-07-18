use serde::{Deserialize, Serialize};

pub mod account;
pub mod bar;
pub mod order;
pub mod position;
pub mod trade;

#[derive(Debug, Clone, Deserialize)]
pub struct Clock {
    pub is_open: bool,
}
