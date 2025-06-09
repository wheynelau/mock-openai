use once_cell::sync::Lazy;
use std::fs;

// Initialise all the global variables here
pub static TOKENIZED_OUTPUT: Lazy<Vec<String>> = Lazy::new(init_string);
pub static MAX_TOKENS: Lazy<usize> = Lazy::new(|| TOKENIZED_OUTPUT.len());
pub static MAX_OUTPUT: Lazy<String> = Lazy::new(|| TOKENIZED_OUTPUT.join(""));

fn raw_string() -> String {
    let link = "https://raw.githubusercontent.com/martin-gorner/tensorflow-rnn-shakespeare/refs/heads/master/shakespeare/sonnets.txt";
    fs::read_to_string("assets/sonnets.txt").unwrap_or_else(|_| {
        panic!(
            "File not found, please download it directly from {} and place it at assets/sonnet.txt",
            link
        )
    })
}

fn init_string() -> Vec<String> {
    let contents = raw_string();
    if let Ok(tokenizer) =
        tokenizers::Tokenizer::from_pretrained("NousResearch/DeepHermes-3-Llama-3-8B-Preview", None)
    {
        log::info!("Loaded the tokenizer");
        let tokens = tokenizer
            .encode(contents, false)
            .unwrap()
            .get_ids()
            .to_vec();
        tokens
            .iter()
            .map(|token| tokenizer.decode(&[*token], true).unwrap())
            .collect()
    } else {
        log::error!("Failed to load the tokenizer, falling back to a simple whitespace tokenizer");
        contents.split_whitespace().map(|s| s.to_string()).collect()
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
