use once_cell::sync::Lazy;
use std::fs;

// Initialise all the global variables here
pub static TOKENIZED_OUTPUT: Lazy<Vec<String>> = Lazy::new(init_string);
pub static MAX_TOKENS: Lazy<usize> = Lazy::new(|| TOKENIZED_OUTPUT.len());

fn raw_string() -> String {
    fs::read_to_string("assets/sonnet.txt").expect("Should have been able to read the file")
}

fn init_string() -> Vec<String> {
    let contents = raw_string();
    // do we want to allow control here?
    // Such as the ability to switch between different tokenizers?
    // TODO: Add a configuration option to allow for different tokenizers
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
            .collect::<Vec<String>>()
    } else {
        // fall back to a simple whitespace tokenizer
        log::error!("Failed to load the tokenizer, falling back to a simple whitespace tokenizer");
        contents.split_whitespace().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_init_string() {
        // to be implemented
        let tokenizer = tokenizers::Tokenizer::from_pretrained(
            "NousResearch/DeepHermes-3-Llama-3-8B-Preview",
            None,
        )
        .expect("Should have been able to load the tokenizer");
        let contents = "This is a test";
        let tokens = tokenizer
            .encode(contents, false)
            .unwrap()
            .get_ids()
            .to_vec();
        let clean_tokens: Vec<String> = tokens
            .iter()
            .map(|token| tokenizer.decode(&[*token], true).unwrap())
            .collect();
        println!("{:?}", clean_tokens);
    }
}
