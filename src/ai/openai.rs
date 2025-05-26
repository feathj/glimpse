use async_openai::{Client, config::OpenAIConfig};

pub async fn openai_client() -> Client<OpenAIConfig> {
    let config = OpenAIConfig::default();
    return Client::with_config(config);
}

