use std::error::Error;
use std::result::Result;
use glob::glob;

use crate::processing::metadata;
use crate::ai::{vision, llm, embedding};
use crate::processing::args;

async fn tag_person(reference_file: &str, files: Vec<String>, person_name: &str, confidence: f32) -> Result<(), Box<dyn Error>> {
    let total = files.len();
    let mut count = 0;
    for file in files {
        count += 1;
        println!("{} / {}: {}", count, total, file);
        // check if file exists
        if !std::path::Path::new(&file).exists() {
            println!("File does not exist: {}", file);
            continue;
        }

        match metadata::get_metadata(&file) {
            Ok(mut metadata) => {
                if metadata.people.contains(&person_name.to_string()) {
                    println!("{} is already tagged in {}", person_name, file);
                } else {
                    match vision::compare_faces(reference_file, &file).await {
                        Ok(similarity) => {
                            if similarity >= confidence { // TODO: check if this is right threshold?
                                metadata.people.push(person_name.to_string());
                                if let Err(e) = metadata::write_metadata(&file, metadata).await {
                                    println!("Failed to write metadata for {}: {:?}", file, e);
                                } else {
                                    println!("Tagged {} in {}", person_name, file);
                                }
                            }
                        }
                        Err(e) => println!("Failed to compare faces for {}: {:?}", file, e),
                    }
                }
            }
            Err(e) => println!("Failed to get metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn show_metadata(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
        match metadata::get_metadata(&file) {
            Ok(metadata) => println!("{}: {:?}", file, metadata),
            Err(e) => println!("Failed to get metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn clear_metadata(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
        match metadata::write_metadata(&file, metadata::PhotoMeta {
            people: vec![],
            description: "".to_string(),
            description_embedding: vec![],
            tags: vec![],
        }).await {
            Ok(_) => println!("Cleared metadata for {}", file),
            Err(e) => println!("Failed to clear metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn tag_description(files: Vec<String>, overwrite: bool, prompt: &str) -> Result<(), Box<dyn Error>> {
    let total = files.len();
    let mut count = 0;
    for file in files {
        count += 1;
        println!("{} / {}: {}", count, total, file);

        // Load original metadata
        let mut metadata = match metadata::get_metadata(&file) {
            Ok(metadata) => metadata,
            Err(e) => {
                println!("Failed to get metadata for {}: {:?}", file, e);
                continue;
            }
        };

        if metadata.description != "" && !overwrite {
            println!("Description already exists for {}", file);
            continue;
        }

        // Get description from AI, including additional context (people in the photo)
        let description = match llm::describe_image(&file, &metadata.description_context(), &prompt).await {
            Ok(description) => description,
            Err(e) => {
                println!("Failed to describe image for {}: {:?}", file, e);
                continue;
            }
        };
        // Now generate embedding for the description
        let description_embedding = match embedding::generate_embedding(description.clone()).await {
            Ok(embedding) => embedding,
            Err(e) => {
                println!("Failed to generate embedding for {}: {:?}", file, e);
                continue;
            }
        };
        metadata.description = description;
        metadata.description_embedding = description_embedding;

        // Write updated metadata
        match metadata::write_metadata(&file, metadata).await {
            Ok(_) => println!("Tagged description for {}", file),
            Err(e) => println!("Failed to write metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn tag(files: Vec<String>, tags: &Vec<String>, overwrite: bool) -> Result<(), Box<dyn Error>> {
    let total = files.len();
    let mut count = 0;
    for file in files {
        count += 1;
        println!("{} / {}: {}", count, total, file);

        // Load original metadata
        let mut metadata = match metadata::get_metadata(&file) {
            Ok(metadata) => metadata,
            Err(e) => {
                println!("Failed to get metadata for {}: {:?}", file, e);
                continue;
            }
        };

        if overwrite {
            metadata.tags = vec![];
        }

        // Get tag from AI
        let tag = match llm::tag_metadata(&metadata, tags).await {
            Ok(tag) => tag,
            Err(e) => {
                println!("Failed to tag from metadata for {}: {:?}", file, e);
                continue;
            }
        };
        // Check if tag is already in metadata
        if metadata.tags.contains(&tag) {
            println!("Tag already exists for {}", file);
            continue;
        }
        metadata.tags.push(tag);

        // Write updated metadata
        match metadata::write_metadata(&file, metadata).await {
            Ok(_) => println!("Tagged metadata for {}", file),
            Err(e) => println!("Failed to write metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}


async fn find_similar(reference_file: &str, files: Vec<String>) -> Result<(), Box<dyn Error>> {
    // Load original metadata
    let reference_metadata = metadata::get_metadata(&reference_file)?;

    // Now load metadata for all other files
    let files_metadata: Vec<(String, metadata::PhotoMeta)> = metadata::get_metadata_list(&files)
        .into_iter()
        .flatten()
        .collect();

    // Now sort by cosine similarity
    let mut similarity_list: Vec<(String, f64)> = vec![];
    for (file, metadata) in files_metadata {
        let similarity = embedding::cosine_similarity(&reference_metadata.description_embedding, &metadata.description_embedding);
        similarity_list.push((file, similarity));
    }
    similarity_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Print
    for (file, similarity) in similarity_list.iter() {
        println!("{}: {}", file, similarity);
    }

    Ok(())
}

pub async fn run(args: &args::Args) -> Result<(), Box<dyn Error>> {
    // expand glob pattern in files
    let files = glob(&args.files)?
        .filter_map(Result::ok)  // Handle errors for individual paths
        .filter_map(|path| path.to_str().map(String::from))  // Convert to strings
        .collect();

    let tags: Vec<String> = args.tags.split(',')
    .map(|tag| tag.trim().to_string())  // Split and trim whitespace
    .collect();

    if args.action == "tag-person" {
        return tag_person(&args.reference_file, files, &args.person_name, args.confidence).await;
    } else if args.action == "tag-description" {
        return tag_description(files, args.overwrite, &args.prompt).await;
    } else if args.action == "tag" {
        return tag(files, &tags, args.overwrite).await;
    } else if args.action == "clear-metadata" {
        return clear_metadata(files).await;
    } else if args.action == "find-similar" {
        return find_similar(&args.reference_file, files).await;
    } else if args.action == "show-metadata" {
        return show_metadata(files).await;
    } else {
        println!("Unknown action: {}", args.action);
    }
    return Ok(());
}