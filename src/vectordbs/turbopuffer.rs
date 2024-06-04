use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use indexify_internal_api::{ContentMetadata, Embedding};
use turbopuffer_client::{Client, NamespacedClient};

use serde::{json, Deserialize, Serialize};

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

// TODO: Figure out how to convert IndexifyPayload to TurboPufferPayload
fn extract_metadata_from_attributes(
    attributes: HashMap<String, Value>,
) -> Result<(HashMap<String, serde_json::Value>, IndexifyPayload)> {
    let attributes: serde_json::Value =
        serde_json::to_value(attributes).map_err(|e| anyhow!("{}", e.to_string()))?;
    let mut attributes: HashMap<String, serde_json::Value> =
        serde_json::from_value(attributes.clone()).map_err(|e| anyhow!(e.to_string()))?;
    let indexify_payload = attributes
        .remove("indexify_payload")
        .ok_or(anyhow!("no indexify system payload found"))?;
    let indexify_payload: IndexifyPayload =
        serde_json::from_value(indexify_payload.clone()).map_err(|e| anyhow!(e));
    Ok((attributes, indexify_payload))
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
}

#[async_trait]
impl VectorDb for TurboPuffer {
    fn name(&self) -> String {
        "turbopuffer".to_string()
    }

    #[tracing::instrument]
    async fn create_index(&self, index: CreateIndexParams) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument]
    async fn add_embedding(&self, index: &str, chunks: Vec<VectorChunk>) -> Result<()> {
        let distance = "".into();

        let body = json!({
          "ids": chunks.iter().map(|c| c.content_id.clone()).collect::<Vec<String>>(),
          "vectors": chunks.iter().map(|c| c.embedding.clone()).collect::<Vec<Embedding>(),
          "attributes": chunks.iter().map(|c| c.labels.clone()).collect::<Vec<ContentMetadata>(),
          "distance": "cosine_distance"
        });

        let client = self.create_client()?;

        let ns = client.namespace(index);

        let res = ns
            .upsert(body)
            .await
            .map_err(|e| anyhow!("Failed to upsert: {}", e.to_string()))?;

        Ok(())
    }

    #[tracing::instrument]
    async fn remove_embedding(&self, index: &str, content_id: &str) -> Result<()> {
        todo!()
    }

    #[tracing::instrument]
    async fn get_points(&self, index: &str, content_ids: Vec<String>) -> Result<Vec<VectorChunk>> {
        let client = self.create_client()?;
        let ns = client.namespace(index);

        let body = json!({
            "ids": content_ids
            "include_vectors": true,
        });

        let res = ns
            .query(body)
            .await
            .map_err(|e| anyhow!("Failed to read index: {}", e.to_string()))?;

        let mut chunks: Vec<VectorChunk> = Vec::new();

        for doc in res.vectors {
            let (metadata, indexify_payload) = extract_metadata_from_attributes(doc.attributes)
                .map_err(|e| anyhow!("Unable to get points: {}", e.to_string()));
            let embedding = doc.vector;

            let chunk = VectorChunk {
                metadata,
                embedding,
                content_id: indexify_payload.content_id,
                root_content_metadata: indexify_payload.root_content_metadata,
                content_metadata: indexify_payload.content_metadata,
            };

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    #[tracing::instrument]
    async fn update_metadata(
        &self,
        index: &str,
        content_id: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        todo!()
    }

    async fn search(
        &self,
        index: String,
        query_embedding: Vec<f32>,
        k: u64,
        filters: Vec<Filter>,
    ) -> Result<Vec<SearchResult>> {
        if !filters.is_empty() {
            // TOOD: Create filter struct
            unimplemented!();
        }

        let client = self.create_client()?;

        let ns = client.namespace(&index);

        let query = json!({
            "top_k": k,
            "vector": query_embedding,
            "distance_metric": "cosine_distance",
            "filter": filter,
            "include_vectors": false
            "include_attributes": true,
        });

        let res = ns
            .query(&query)
            .await
            .map_err(|e| anyhow!("Failed to search: {}", e.to_string()))?;

        let mut documents: Vec<SearchResult> = Vec::new();

        for doc in res.vectors {
            let (metadata, indexify_payload) = extract_metadata_from_attributes(doc.attributes)?;
            // Only f32
            let embedding = doc.vector.unwrap();

            documents.push(SearchResult {
                metadata,
                content_id: indexify_payload.content_id,
                confidence_score: doc.dist,
                content_metadata: indexify_payload.content_metadata,
                root_content_metadata: indexify_payload.root_content_metadata,
            })
        }

        Ok(documents)
    }

    async fn drop_index(&self, index: &str) -> Result<()> {
        let client = self.create_client()?;

        let res = client
            .namespace(&index)
            .delete()
            .await
            .map_err(|e| anyhow!("unable to drop {}, err: {}", index, e.to_string()));

        Ok(())
    }

    async fn num_vectors(&self, index: &str) -> Result<u64> {
        todo!()
    }
}
