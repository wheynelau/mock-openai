use once_cell::sync::Lazy;
use std::fs;
use std::path::Path;

// Initialise all the global variables here
pub static TOKENIZED_OUTPUT: Lazy<Vec<String>> = Lazy::new(init_string);
pub static MAX_TOKENS: Lazy<usize> = Lazy::new(|| TOKENIZED_OUTPUT.len());
pub static MAX_OUTPUT: Lazy<String> = Lazy::new(|| TOKENIZED_OUTPUT.join(""));

pub async fn download_sonnets() -> Result<(), Box<dyn std::error::Error>> {
    let link = "https://raw.githubusercontent.com/martin-gorner/tensorflow-rnn-shakespeare/refs/heads/master/shakespeare/sonnets.txt";
    let path = "assets/sonnets.txt";

    log::info!("Starting sonnets download process");
    log::debug!("Download URL: {}", link);
    log::debug!("Target path: {}", path);

    // Create assets directory if it doesn't exist
    if !Path::new("assets").exists() {
        log::info!("Creating assets directory");
        fs::create_dir("assets")?;
        log::debug!("Assets directory created successfully");
    }

    // Check if file already exists
    if Path::new(path).exists() {
        log::info!("sonnets.txt already exists at {}", path);
        println!("sonnets.txt already exists at {}", path);
        return Ok(());
    }

    log::info!("Downloading sonnets.txt from {}...", link);
    println!("Downloading sonnets.txt from {}...", link);

    let response = reqwest::get(link).await?;
    log::debug!("Received response with status: {}", response.status());

    if !response.status().is_success() {
        let error_msg = format!("Failed to download: HTTP {}", response.status());
        log::error!("Download failed: {}", error_msg);
        return Err(error_msg.into());
    }

    let content = response.text().await?;
    log::debug!("Downloaded {} bytes of content", content.len());

    fs::write(path, content)?;
    log::info!("Successfully wrote sonnets.txt to {}", path);

    println!("Successfully downloaded sonnets.txt to {}", path);
    Ok(())
}

fn raw_string() -> String {
    let link = "https://raw.githubusercontent.com/martin-gorner/tensorflow-rnn-shakespeare/refs/heads/master/shakespeare/sonnets.txt";
    fs::read_to_string("assets/sonnets.txt").unwrap_or_else(|_| {
        panic!(
            "File not found, please download it directly from {} and place it at assets/sonnets.txt\nYou can also run the binary with --download-sonnets flag to download it automatically.",
            link
        )
    })
}

fn init_string() -> Vec<String> {
    log::info!("Initializing tokenized output from sonnets");
    let contents = raw_string();
    log::info!("Loaded {} characters from sonnets.txt", contents.len());

    if let Ok(tokenizer) =
        tokenizers::Tokenizer::from_pretrained("NousResearch/DeepHermes-3-Llama-3-8B-Preview", None)
    {
        log::info!(
            "Successfully loaded the tokenizer: NousResearch/DeepHermes-3-Llama-3-8B-Preview"
        );
        let tokens = tokenizer
            .encode(contents, false)
            .unwrap()
            .get_ids()
            .to_vec();
        log::info!("Encoded text into {} tokens", tokens.len());

        let decoded_tokens: Vec<String> = tokens
            .iter()
            .map(|token| tokenizer.decode(&[*token], true).unwrap())
            .collect();
        log::info!("Successfully decoded {} tokens", decoded_tokens.len());
        decoded_tokens
    } else {
        log::warn!("Failed to load the tokenizer, falling back to a simple whitespace tokenizer");
        let fallback_tokens: Vec<String> =
            contents.split_whitespace().map(|s| s.to_string()).collect();
        log::info!(
            "Using fallback tokenizer with {} tokens",
            fallback_tokens.len()
        );
        fallback_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_init_string_template() {
        let baseline = raw_string();
        let template = init_string().join("");
        assert_eq!(baseline, template);
    }
}
