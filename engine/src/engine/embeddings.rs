use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub path: String,
    pub embedding: Option<Vec<f32>>,
}

pub async fn embed_text(text: &str) -> anyhow::Result<Vec<f32>> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
    
    let client = reqwest::Client::new();
    
    let request = EmbeddingRequest {
        input: text.to_string(),
        model: "text-embedding-3-small".to_string(),
    };
    
    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("OpenAI embeddings error: {}", error_text));
    }
    
    let embedding_response: EmbeddingResponse = response.json().await?;
    
    embedding_response
        .data
        .first()
        .map(|data| data.embedding.clone())
        .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

pub async fn scan_local_files() -> anyhow::Result<Vec<Document>> {
    let mut documents = Vec::new();
    let extensions = ["rs", "md", "toml", "yaml", "json"];
    
    for entry in walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if extensions.contains(&ext.to_str().unwrap_or("")) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    // Chunk into 512-token segments (roughly 2000 chars)
                    for (i, chunk) in content.chars().collect::<Vec<_>>()
                        .chunks(2000)
                        .enumerate()
                    {
                        let chunk_content: String = chunk.iter().collect();
                        if chunk_content.trim().len() > 50 { // Skip tiny chunks
                            documents.push(Document {
                                id: format!("{}#{}", path.display(), i),
                                content: chunk_content,
                                path: path.to_string_lossy().to_string(),
                                embedding: None,
                            });
                        }
                    }
                }
            }
        }
    }
    
    Ok(documents)
}

pub async fn build_embeddings_index() -> anyhow::Result<Vec<Document>> {
    let mut documents = scan_local_files().await?;
    
    for doc in &mut documents {
        match embed_text(&doc.content).await {
            Ok(embedding) => doc.embedding = Some(embedding),
            Err(e) => {
                tracing::warn!("Failed to embed {}: {}", doc.id, e);
            }
        }
        // Rate limit: small delay between API calls
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    Ok(documents)
}
