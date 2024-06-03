use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use indexify_internal_api::ContentMetadata;
use turbo_client::client::{TurboClient, TurboClientConfig};

use serde::{Deserialize, Serialize};

use super::{CreateIndexParams, VectorDb};
use crate::{
    server_config::TurboClientConfig,
    vectordbs::{FilterOperator, IndexDistance, SearchResult, VectorChunk},
};

#[derive(Debug)]
pub struct TurboPuffer {
    turbo_config: TurboClientConfig,
}

impl TurboPuffer {
    pub fn new(turbo_config: TurboClientConfig) -> Self {
        TurboPuffer { turbo_config }
    }
}
