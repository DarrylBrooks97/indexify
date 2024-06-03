use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use indexify_internal_api::ContentMetadata;
use turbopuffer_client::Client;

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
    pub fn new(config: TurboClientConfig) -> TurboPuffer {
        Self {
            turbo_config: config,
        }
    }

    pub fn create_client(&self) -> Result<TurboClient> {
        let client_config = TurboClientConfig::new(&self.turbo_config.api_key);
        let client = TurboClient::new(Some(client_config))
            .map_err(|e| anyhow!("Failed to create TurboClient: {}", e))?;
        Ok(client)
    }

    fn get_namespace(&self, ns: str) -> Result<NamespacedClient> {
        let client = self.create_client()?;
        let ns_client = client
            .get_namespace(ns)
            .map_err(|e| anyhow!("Failed to get namespace {}: {}", ns, e))?;
        Ok(ns_client)
    }

    pub async fn list_namespaces() -> Result<Vec<String>> {
        unimplemented!()
    }

    pub async fn upsert(&self) -> Result<()> {
        unimplemented!()
    }

    pub async fn delete_namespace(&self) -> Result<()> {
        unimplemented!()
    }
}
