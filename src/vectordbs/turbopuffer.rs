use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use indexify_internal_api::{ContentMetadata, Embedding};
use turbopuffer_client::Client;

use serde::{Deserialize, Serialize};

use super::{CreateIndexParams, VectorDb};
use crate::{
    server_config::TurboClientConfig,
    vectordbs::{FilterOperator, IndexDistance, SearchResult, VectorChunk},
};

fn hex_to_u64(hex: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(hex, 16)
}

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

    pub fn create_client(&self) -> Result<Client> {
        let client_config = TurboClientConfig::new(&self.turbo_config.api_key);
        let client = Client::new(Some(client_config));

        Ok(client)
    }

    pub async fn upsert(&self, namespace: String, chunks: Vec<VectorChunk>) -> Result<()> {
        let client = self.create_client()?;

        let ns = client.namespace(&namespace);
        let mut data = None;

        let ids = chunks
            .iter()
            .map(|c| c.content_id.clone())
            .map(|id| hex_to_u64(&id).unwrap_or(0))
            .collect::<Vec<String>>();

        let vectors = chunks
            .iter()
            .map(|c| c.embedding.clone())
            .collect::<Vec<Embedding>>();

        // Will probably have to change the values to be a hashmap
        let attributes = chunks
            .iter()
            .map(|c| c.metadata.clone())
            .collect::<Vec<ContentMetadata>>();

        data = serde::json!({
            "ids": ids,
            "vectors": vectors,
            "attributes": attributes,
        });

        let res = ns.upsert(&data).await;

        match res {
            UpsertResponse => println!("Upserted data!"),
            Error => println!("Failed to upsert data: {}", e),
        };

        Ok(())
    }

    pub async fn search(
        &self,
        namespace: String,
        query_embedding: Vec<f32>,
        k: u64,
        filters: Vec<super::Filter>,
    ) {
        if !filters.is_empty() {
            // TOOD: Create filter struct
            unimplemented!();
        }

        let client = self.create_client()?;

        let ns = client.namespace(&namespace);

        let query = serde::json!({
            "query": query_embedding,
            "k": k,
            "filter": filter,
        });

        let res = ns.query(&query).await;

        match res {
            QueryResponse => println!("Search result: {:?}", res),
            Error => println!("Failed to search"),
        }
    }

    pub async fn delete_namespace(&self, namespace: String) -> Result<()> {
        let client = self.create_client()?;

        let res = client.namespace(&namespace).delete().await;

        match res {
            DeleteResponse => println!("Deleted namespace: {}", namespace),
            Error => println!("Failed to delete namespace: {}", namespace),
        }
    }

    pub async fn update(&self) {
        unimplemented!();
    }
}
