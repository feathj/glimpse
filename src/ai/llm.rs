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

pub async fn describe_image(file_path: &str, _image_metadata: &PhotoMeta, prompt: &str) -> Result<String, Box<dyn Error>> {
    let content_text = if prompt.is_empty() {
        //let people = image_metadata.people.iter().fold("".to_string(), |acc, person| format!("{}<people>{}</people>", acc, person));
        // Claude appears to ignore people information provided in prompt TODO: Figure this out
        format!(
            "
            You are an expert image analyst providing detailed visual descriptions. Please describe the provided image comprehensively, focusing on:

            1. People in the scene:
                - Number of people
                - Their actions, interactions, and positioning
                - Notable expressions and body language
                - Distinctive clothing or accessories
                - Group dynamics if multiple people are present

            2. Setting and context:
                - Location type (indoor/outdoor, specific setting)
                - Event or activity type (if apparent)
                - Time period indicators
                - Overall mood/atmosphere

            3. Key visual details for categorization:
                - Composition style
                - Lighting conditions
                - Notable objects or elements
                - Any unique or distinguishing features

            Please emphasize details that would be useful for future categorization or searching.
            "
        )
    } else {
        prompt.to_string()
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
    let tagged_people = metadata.people.iter().fold("".to_string(), |acc, person| format!("{}<person>{}</person>", acc, person));

    let prompt = format!(
        "
        You are acting as an expert labeling system for an image.
        You will be given a list of possible labels to chose from.
        You will chose exactly one from that list.
        You will chose the label based on the provided description and people tagged in the image.
        Return the label only in <label></label>.

        <people>{}</people>
        <description>{}</description>
        <labels>{}</labels>", tagged_people, metadata.description, labels
    );
    let result = converse(&prompt).await
    .map(|response| response.replace("<label>", "").replace("</label>", ""));

    // Check if the response is in the list of tags
    match result {
        Ok(tag) => {
            if tags.contains(&tag) {
                Ok(tag)
            } else {
                Ok("".to_string())
            }
        }
        Err(e) => Err(e)
    }
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