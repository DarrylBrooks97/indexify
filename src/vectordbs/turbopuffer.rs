use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use indexify_internal_api::ContentMetadata;
use turbopuffer_client::Client;


use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use super::{CreateIndexParams, VectorDb};
use crate::{
    server_config::TurboClientConfig,
    vectordbs::{SearchResult, VectorChunk},
};

#[derive(Debug, Serialize, Deserialize)]
struct IndexifyPayload {
    pub content_metadata: ContentMetadata,
    pub root_content_metadata: Option<ContentMetadata>,
}

impl IndexifyPayload {
    pub fn new(
        content_metadata: ContentMetadata,
        root_content_metadata: Option<ContentMetadata>,
    ) -> Self {
        Self {
            content_metadata,
            root_content_metadata,
        }
    }
}

fn hex_to_u64(hex: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(hex, 16)
}

fn extract_metadata_from_attributes(
    attributes: HashMap<String, Value>
) -> Result<(HashMap<String, serde_json::Value>, IndexifyPayload)> {
    
    let value = serde_json::to_value(attributes).map_err(|e| anyhow!("{}", e.to_string()))?;
    let payload: HashMap<String, Value>= serde_json::from_value(value.clone()).map_err(|e| anyhow!("{}", e.to_string()))?;
    let indexify_payload: IndexifyPayload =  serde_json::from_value(value.clone()).map_err(|e| anyhow!("{}", e.to_string()))?;

    Ok((payload, indexify_payload))
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
        let client = Client::new(&self.turbo_config.api_key);

        Ok(client)
    }
}

#[async_trait]
impl VectorDb for TurboPuffer {
    fn name(&self) -> String {
        "turbopuffer".to_string()
    }

    #[tracing::instrument]
    async fn create_index(&self, _index: CreateIndexParams) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument]
    async fn add_embedding(&self, index: &str, chunks: Vec<VectorChunk>) -> Result<()> {

        let payload: Vec<serde_json::Value> = chunks
            .iter()
            .map(|chunk| {
                let ids = vec![chunk.content_id.clone()];

                json!({
                    "ids": ids,
                    "vectors": chunk.embedding,
                    "attributes": &chunk.content_metadata,
                })
            })
            .collect();

        let client = self.create_client()?;

        let ns = client.namespace(index);

        for payload in payload.iter() {
             ns
                .upsert(payload)
                .await
                .map_err(|e| anyhow!("Failed to upsert: {}", e.to_string()));
        }

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
            "ids": content_ids,
            "include_vectors": true,
        });

        let res = ns
            .query(&body)
            .await
            .map_err(|e| anyhow!("Failed to read index: {}", e.to_string()))?;

        let mut chunks: Vec<VectorChunk> = Vec::new();

        for doc in res.vectors {
            let content_id = doc.id.to_string();
            let embedding = doc.vector.unwrap();
            let metadata = doc.attributes.unwrap();

            let (payload, indexify_payload) = extract_metadata_from_attributes(metadata)?;

            let chunk = VectorChunk {
                embedding,
                content_id,
                metadata: payload,
                content_metadata: indexify_payload.content_metadata,
                root_content_metadata: indexify_payload.root_content_metadata,
            };

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    #[tracing::instrument]
    async fn update_metadata(
        &self,
        index: &str,
        _content_id: String,
        _metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        todo!()
    }

    async fn search(
        &self,
        index: String,
        query_embedding: Vec<f32>,
        k: u64,
        filters: Vec<super::Filter>,
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
            // "filters": filters,
            "include_vectors": false,
            "include_attributes": true,
        });

        let res = ns
            .query(&query)
            .await
            .map_err(|e| anyhow!("Failed to search: {}", e.to_string()))?;

        let mut documents: Vec<SearchResult> = Vec::new();

        for doc in res.vectors {
            let attributes = doc.attributes.unwrap();
            let ( payload, indexify_payload ) = extract_metadata_from_attributes(attributes)?;

            documents.push(SearchResult {
                content_id: doc.id.to_string(),
                metadata: payload,
                confidence_score: doc.dist,
                content_metadata: indexify_payload.content_metadata,
                root_content_metadata: indexify_payload.root_content_metadata,
            })
        }

        Ok(documents)
    }

    async fn drop_index(&self, index: &str) -> Result<()> {
        let client = self.create_client()?;

        client
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