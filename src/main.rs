use std::error::Error;
use std::path::Path;
use std::fs;
use hound::WavReader;
use clap::{Parser, Subcommand};
use singerprint::{
    audio_processor::AudioProcessor,
    fingerprint::AudioFingerprint,
    matcher::FingerprintMatcher,
};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "singerprint")]
#[command(about = "An audio fingerprinting tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate fingerprint from an audio file
    Generate {
        /// Input audio file path
        #[arg(short, long)]
        input: String,
        
        /// Output file to save the fingerprint
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Match an audio file against the database
    Match {
        /// Audio file to match
        #[arg(short, long)]
        input: String,
        
        /// Database file containing fingerprints
        #[arg(short, long)]
        database: String,
    },
    /// Add a fingerprint to the database
    Add {
        /// Audio file to fingerprint
        #[arg(short, long)]
        input: String,
        
        /// Name to associate with the fingerprint
        #[arg(short, long)]
        name: String,
        
        /// Database file to add the fingerprint to
        #[arg(short, long)]
        database: String,
    },
}

fn save_fingerprint(fingerprint: &AudioFingerprint, path: &str) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(fingerprint)?;
    fs::write(path, json)?;
    Ok(())
}

fn load_database(path: &str) -> Result<HashMap<String, AudioFingerprint>, Box<dyn Error>> {
    if Path::new(path).exists() {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(HashMap::new())
    }
}

fn save_database(database: &HashMap<String, AudioFingerprint>, path: &str) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(database)?;
    fs::write(path, json)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let processor = AudioProcessor::new(44100);
    
    match cli.command {
        Commands::Generate { input, output } => {
            let fingerprint = process_file(&input, &processor)?;
            if let Some(output_path) = output {
                save_fingerprint(&fingerprint, &output_path)?;
                println!("Generated fingerprint saved to: {}", output_path);
            } else {
                println!("Fingerprint generated for: {}", input);
                println!("{} peaks found", fingerprint.peaks.len());
            }
        }
        
        Commands::Match { input, database } => {
            let database_content = load_database(&database)?;
            let mut matcher = FingerprintMatcher::new();
            
            // Add all fingerprints from the database
            for (name, fp) in database_content {
                matcher.add_fingerprint(&name, fp);
            }
            
            let query_fingerprint = process_file(&input, &processor)?;
            
            if let Some(match_name) = matcher.find_match(&query_fingerprint) {
                println!("Match found: {}", match_name);
            } else {
                println!("No match found");
            }
        }
        
        Commands::Add { input, name, database } => {
            let mut database_content = load_database(&database)?;
            let fingerprint = process_file(&input, &processor)?;
            
            database_content.insert(name.clone(), fingerprint);
            save_database(&database_content, &database)?;
            
            println!("Added fingerprint for '{}' to database: {}", name, database);
        }
    }
    
    Ok(())
}

fn process_file(path: &str, processor: &AudioProcessor) -> Result<AudioFingerprint, Box<dyn Error>> {
    let mut reader = WavReader::open(Path::new(path))?;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();
    
    processor.process_audio(&samples)
}