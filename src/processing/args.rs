use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    // Core arguments
    #[arg(short, long, required = true)]
    pub action: String,
    #[clap(short, long, required = true)]
    pub files: String,
    #[clap(short, long, default_value = "10")]
    pub top: u32,
    #[clap(short, long, default_value = "bedrock")]
    pub provider: String,
    // Tagging arguments
    #[clap(short, long, required_if_eq("action", "tag-person"), default_value = "")]
    pub person_name: String,
    #[clap(short, long, required_if_eq("action", "tag-person"), default_value = "")]
    pub reference_file: String,
    #[clap(short, long, default_value = "85.0")]
    pub confidence: f32,
    #[clap(short, long, action)]
    pub overwrite: bool,
    #[clap(short, long, default_value = "")]
    pub description: String,
    #[clap(short, long, required_if_eq("action", "tag"), default_value = "")]
    pub tags: String,
    #[clap(short, long, required_if_eq("action", "sort-by-tag"), default_value = "")]
    pub output_directory: String,
    #[arg(short, long, required = false, default_value = "")]
    pub prompt: String
}