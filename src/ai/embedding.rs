use std::error::Error;

use crate::ai::bedrock::bedrock_client;
use async_openai::types::CreateEmbeddingRequestArgs;
use aws_sdk_bedrockruntime::primitives::Blob;

const BEDROCK_MODEL_ID: &str = "cohere.embed-english-v3";
const OPENAI_MODEL_ID: &str = "text-embedding-3-large";

pub async fn generate_embedding(provider: &str, text: String) -> (String, Result<Vec<f64>, Box<dyn Error>>) {
    match provider {
        "openai" => {
            let embeddings = generate_embedding_openai(text).await;
            return (OPENAI_MODEL_ID.to_string(), embeddings);
        }
        "bedrock" => {
            let embeddings = generate_embedding_bedrock(text).await;
            return (BEDROCK_MODEL_ID.to_string(), embeddings);
        }
        _ => {
            return (
                provider.to_string(),
                Err(Box::from("Invalid provider")),
            );
        }
    }
}
async fn generate_embedding_openai(text: String) -> Result<Vec<f64>, Box<dyn Error>> {
    let openai_client = crate::ai::openai::openai_client().await;
    let request = CreateEmbeddingRequestArgs::default()
        .model(OPENAI_MODEL_ID)
        .input(text)
        .build()?;

    let response = openai_client.embeddings().create(request).await?;

    return Ok(
        response
            .data
            .iter()
            .flat_map(|v| v.embedding.iter())
            .filter_map(|v| Some(*v as f64))
            .collect(),
    );
}
async fn generate_embedding_bedrock(text: String) -> Result<Vec<f64>, Box<dyn Error>> {
    let bedrock_client = bedrock_client().await;
    let resp = bedrock_client.invoke_model()
    .model_id(BEDROCK_MODEL_ID)
    .body(Blob::new(
        serde_json::json!({
            "texts": vec![text],
            "input_type": "search_document" // "search_query"
        }).to_string())
    )
    .send()
    .await;

    let body = String::from_utf8(resp?.body().clone().into_inner())?;
    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    let embeddings = json
        .as_object()
        .unwrap()
        .get("embeddings")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|v| v.as_array().unwrap().iter())
        .filter_map(|v| v.as_f64())
        .collect();

    Ok(embeddings)
}

pub fn cosine_similarity(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let dot_product = a.iter().zip(b.iter()).map(|(a, b)| a * b).sum::<f64>();
    let norm_a = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    dot_product / (norm_a * norm_b)
}