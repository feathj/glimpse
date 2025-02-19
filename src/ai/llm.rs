use std::error::Error;

use aws_sdk_bedrockruntime::{
    operation::converse::ConverseOutput,
    types::{ContentBlock, ConversationRole, Message},
};

use crate::ai::bedrock::bedrock_client;
use crate::ai::bedrock::BedrockConverseError;
use crate::graphics::images::{path_to_bedrock_image_block, resize_temp_image, clear_temp_file};
use crate::processing::metadata::PhotoMeta;

// const MODEL_ID: &str = "anthropic.claude-3-haiku-20240307-v1:0";
const MODEL_ID: &str = "anthropic.claude-3-5-sonnet-20241022-v2:0";

fn get_converse_output_text(output: ConverseOutput) -> Result<String, BedrockConverseError> {
    let text = output
        .output()
        .ok_or("no output")?
        .as_message()
        .map_err(|_| "output not a message")?
        .content()
        .first()
        .ok_or("no content in message")?
        .as_text()
        .map_err(|_| "content is not text")?
        .to_string();
    Ok(text)
}

pub async fn describe_image(file_path: &str, additional_context: &str, prompt: &str) -> Result<String, Box<dyn Error>> {
    // Inject the prompt if provided, otherwise use a generic
    let content_text = if prompt.is_empty() {
        format!("Describe the following image. Here is some additional context: {}", additional_context)
    } else {
        format!("{}. Here is some additional context: {}", prompt, additional_context)
    };

    let tmp_file_path = resize_temp_image(file_path, 1000)?; // TODO: make a more scientific decision on the resizes
    let message_user = Message::builder()
        .role(ConversationRole::User)
        .content(ContentBlock::Text(content_text.to_string()))
        .content(ContentBlock::Image(path_to_bedrock_image_block(&tmp_file_path)?))
        .build()?;
    clear_temp_file(&tmp_file_path)?;

    let bedrock_client = bedrock_client().await;
    let response = bedrock_client
        .converse()
        .messages(message_user)
        .model_id(MODEL_ID)
        .send()
        .await;

    match response {
        Ok(output) => {
            let text = get_converse_output_text(output)?;
            Ok(text)
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn tag_metadata(metadata: &PhotoMeta, tags: &Vec<String>) -> Result<String, Box<dyn Error>> {
    let labels = tags.iter().fold("".to_string(), |acc, tag| format!("{}<label>{}</label>", acc, tag));

    let prompt = format!(
        "
        You are acting as an expert labeling system for a photo.
        You will be given a list of possible labels to chose from.
        You will chose exactly one from that list.
        You will chose the label based on the provided description and people tagged in the photo.
        Return the label only in <label></label>.

        <people>{:?}</people>
        <description>{}</description>
        <labels>{}</labels>", metadata.people, metadata.description, labels
    );
    println!("Prompt: {}", prompt);
    let result = converse(&prompt).await
    .map(|response| response.replace("<TAG>", "").replace("</TAG>", ""));
    println!("Result: {}", result.as_ref().unwrap());
    result
}

pub async fn converse(content: &str) -> Result<String, Box<dyn Error>> {
    let bedrock_client = bedrock_client().await;
    let response = bedrock_client
        .converse()
        .model_id(MODEL_ID)
        .messages(
            Message::builder()
                .role(ConversationRole::User)
                .content(ContentBlock::Text(content.to_string()))
                .build()
                .map_err(|_| "failed to build message")?,
        )
        .send()
        .await;

    match response {
        Ok(output) => {
            let text = get_converse_output_text(output)?;
            Ok(text)
        }
        Err(e) => Err(Box::new(e)),
    }
}