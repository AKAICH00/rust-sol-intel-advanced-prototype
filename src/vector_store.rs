use crate::types::PatternMetadata;
use anyhow::Result;
use qdrant_client::{
    qdrant::{PointStruct, SearchPointsBuilder, UpsertPointsBuilder},
    Qdrant,
};
use serde_json::Value;
use uuid::Uuid;

/// Client for vector storage and similarity search (e.g., Qdrant).
#[derive(Clone)]
pub struct VectorStore {
    client: Qdrant,
    collection: String,
}

impl VectorStore {
    /// Create a new vector store client for the given HTTP URL.
    pub async fn new(url: &str) -> Result<Self> {
        let client = Qdrant::from_url(url).build()?;
        Ok(Self {
            client,
            collection: "memecoin_patterns".into(),
        })
    }

    /// Insert an embedding with associated metadata.
    pub async fn insert_pattern(
        &self,
        embedding: &[f32],
        metadata: &PatternMetadata,
    ) -> Result<()> {
        let payload: serde_json::Map<String, Value> = serde_json::json!({
            "symbol": metadata.symbol,
            "price": metadata.price,
            "volume": metadata.volume,
        })
        .as_object()
        .cloned()
        .unwrap_or_default();
        let point = PointStruct::new(Uuid::new_v4().to_string(), embedding.to_vec(), payload);
        let upsert = UpsertPointsBuilder::new(&self.collection, vec![point]).build();
        self.client.upsert_points(upsert).await?;
        Ok(())
    }

    /// Find similar embeddings to the query.
    pub async fn find_similar(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(Vec<f32>, f32)>> {
        let result = self
            .client
            .search_points(
                SearchPointsBuilder::new(&self.collection, embedding.to_vec(), limit as u64)
                    .with_payload(true),
            )
            .await?;
        Ok(result
            .result
            .into_iter()
            .map(|p| (Vec::new(), p.score))
            .collect())
    }
}
