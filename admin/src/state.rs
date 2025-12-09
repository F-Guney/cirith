#![allow(dead_code)]

use std::sync::Arc;
// imports
use crate::metrics::Metrics;
use cirith_shared::{auth::AuthValidator, config::Config, storage::Database};

#[derive(Clone)]
pub struct AdminState {
    pub config: Config,
    pub auth_validator: AuthValidator,
    pub metrics: Arc<Metrics>,
    pub database: Arc<Database>,
}
