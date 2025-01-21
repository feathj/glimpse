use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_bedrockruntime::operation::converse::ConverseError;

#[derive(Debug)]
pub struct BedrockConverseError(String);
impl std::fmt::Display for BedrockConverseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Can't invoke model. Reason: {}", self.0)
    }
}
impl std::error::Error for BedrockConverseError {}
impl From<&str> for BedrockConverseError {
    fn from(value: &str) -> Self {
        BedrockConverseError(value.to_string())
    }
}
impl From<&ConverseError> for BedrockConverseError {
    fn from(value: &ConverseError) -> Self {
        BedrockConverseError::from(match value {
            ConverseError::ModelTimeoutException(_) => "Model took too long",
            ConverseError::ModelNotReadyException(_) => "Model is not ready",
            _ => "Unknown",
        })
    }
}

pub async fn bedrock_client() -> aws_sdk_bedrockruntime::Client {
    let bedrock_region = std::env::var("AWS_REGION").ok();

    let bedrock_region_provider = RegionProviderChain::first_try(bedrock_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    let bedrock_shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(bedrock_region_provider)
        .load()
        .await;
    aws_sdk_bedrockruntime::Client::new(&bedrock_shared_config)
}