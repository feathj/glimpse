use std::error::Error;
use std::result::Result;
use std::result::Result::Ok;
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

async fn find_person(files: Vec<String>, person_name: &str) -> Result<(), Box<dyn Error>> {
    let files_metadata = metadata::get_metadata_list(&files)?;
    for (file, metadata) in files_metadata {
        if metadata.people.contains(&person_name.to_string()) {
            println!("{}", file);
        }
    }
    Ok(())
}

async fn show_metadata(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    let files_metadata = metadata::get_metadata_list(&files)?;
    for (file, metadata) in files_metadata {
        println!("{}: {}", file, metadata);
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

async fn sort_by_tag(files: Vec<String>, output_directory: &str) -> Result<(), Box<dyn Error>> {
    let files_metadata = metadata::get_metadata_list(&files)?;
    // Get list of tags
    let mut tags: Vec<String> = vec![];
    for (_file, metadata) in &files_metadata {
        for tag in &metadata.tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
    }

    // Create output directory if it doesn't exist
    match std::fs::create_dir_all(output_directory) {
        Ok(_) => println!("Created output directory: {}", output_directory),
        Err(e) => {
            println!("Failed to create output directory {}: {:?}", output_directory, e);
            return Err(e.into());
        }
    }

    // Create directories for each tag
    for tag in &tags {
        let path = std::path::Path::new(output_directory).join(tag);
        match std::fs::create_dir(&path) {
            Ok(_) => println!("Created directory for tag: {}", tag),
            Err(e) => println!("Failed to create directory for tag {}: {:?}", tag, e),
        }
    }

    // Move files to directories
    for (file, metadata) in files_metadata {
        let tag = &metadata.tags[0];  // TODO: handle multiple tags somehow?
        let path = std::path::Path::new(output_directory).join(tag);
        let new_file = path.join(std::path::Path::new(&file).file_name().unwrap());
        match std::fs::rename(&file, &new_file) {
            Ok(_) => println!("Moved {} to {}", file, new_file.display()),
            Err(e) => println!("Failed to move {} to {}: {:?}", file, new_file.display(), e),
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
        let description = match llm::describe_image(&file, &metadata, &prompt).await {
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
        if tag != "" {
            metadata.tags.push(tag);
        }

        // Write updated metadata
        match metadata::write_metadata(&file, metadata).await {
            Ok(_) => println!("Tagged metadata for {}", file),
            Err(e) => println!("Failed to write metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}


async fn find_similar(reference_file: &str, files: Vec<String>, top: u32) -> Result<(), Box<dyn Error>> {
    // Load original metadata
    let reference_metadata = metadata::get_metadata(&reference_file)?;

    // Now load metadata for all other files
    let files_metadata: Vec<(String, metadata::PhotoMeta)> = match metadata::get_metadata_list(&files) {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("Failed to get metadata list: {:?}", e);
            return Err(e);
        }
    };

    // Now generate similarity list
    let mut similarity_list: Vec<(String, f64)> = vec![];
    for (file, metadata) in files_metadata {
        let similarity = embedding::cosine_similarity(&reference_metadata.description_embedding, &metadata.description_embedding);
        similarity_list.push((file, similarity));
    }
    // Sort by similarity
    similarity_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    if similarity_list.len() > top as usize {
        similarity_list.truncate(top as usize);
    }

    // Print
    for (file, similarity) in similarity_list.iter() {
        println!("{}: {}", file, similarity);
    }

    Ok(())
}

async fn find(files: Vec<String>, description: &str, top: u32) -> Result<(), Box<dyn Error>> {
    // Load metadata for all  files
    let files_metadata: Vec<(String, metadata::PhotoMeta)> = match metadata::get_metadata_list(&files) {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("Failed to get metadata list: {:?}", e);
            return Err(e);
        }
    };

    // Generate embedding for the description
    let description_embedding = embedding::generate_embedding(description.to_string()).await?;

    // Now generate similarity list
    let mut similarity_list: Vec<(String, f64)> = vec![];
    for (file, metadata) in files_metadata {
        let similarity = embedding::cosine_similarity(&metadata.description_embedding, &description_embedding);
        similarity_list.push((file, similarity));
    }
    // Sort by similarity
    similarity_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    if similarity_list.len() > top as usize {
        similarity_list.truncate(top as usize);
    }

    // Print
    for (file, _similarity) in similarity_list.iter() {
        //println!("{}: {}", file, similarity);
        println!("{}", file);
    }

    Ok(())
}

pub async fn run(args: &args::Args) -> Result<(), Box<dyn Error>> {
    // expand glob pattern in files
    let files: Vec<String> = glob(&args.files)?
        .filter_map(Result::ok)  // Handle errors for individual paths
        .filter_map(|path| path.to_str().map(String::from))  // Convert to strings
        .collect();

    let tags: Vec<String> = args.tags.split(',')
        .map(|tag| tag.trim().to_string())  // Split and trim whitespace
        .collect();

    match args.action.as_str() {
        "tag-person" => tag_person(&args.reference_file, files, &args.person_name, args.confidence).await,
        "find-person" => find_person(files, &args.person_name).await,
        "tag-description" => tag_description(files, args.overwrite, &args.prompt).await,
        "tag" => tag(files, &tags, args.overwrite).await,
        "clear-metadata" => clear_metadata(files).await,
        "sort-by-tag" => sort_by_tag(files, &args.output_directory).await,
        "find-similar" => find_similar(&args.reference_file, files, args.top).await,
        "find" => find(files, &args.description, args.top).await,
        "show-metadata" => show_metadata(files).await,
        _ => {
            println!("Unknown action: {}", args.action);
            Ok(())
        }
    }
}