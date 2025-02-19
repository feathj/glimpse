use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, required = true)]
    pub action: String,
    #[clap(short, long, required = true)]
    pub files: String,
    #[clap(short, long, required_if_eq("action", "tag-person"), default_value = "")]
    pub person_name: String,
    #[clap(short, long, required_if_eq("action", "tag-person"), default_value = "")]
    pub reference_file: String,
    #[clap(short, long, default_value = "85.0")]
    pub confidence: f32,
    #[clap(short, long, action)]
    pub overwrite: bool,
    #[clap(short, long, required_if_eq("action", "tag"), default_value = "")]
    pub tags: String,
    #[arg(short, long, required = false, default_value = "")]
    pub prompt: String,
    #[clap(short, long, default_value = "10")]
    pub top: u32
}