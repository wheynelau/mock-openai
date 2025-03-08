use once_cell::sync::Lazy;
use std::fs;

// Initialise all the global variables here
pub static TOKENIZED_OUTPUT: Lazy<Vec<&'static str>> = Lazy::new(init_string);
pub static MAX_TOKENS: Lazy<usize> = Lazy::new(|| TOKENIZED_OUTPUT.len());
pub static MAX_OUTPUT: Lazy<String> = Lazy::new(|| TOKENIZED_OUTPUT.concat());

fn raw_string() -> String {
    let link = "https://raw.githubusercontent.com/martin-gorner/tensorflow-rnn-shakespeare/refs/heads/master/shakespeare/sonnets.txt";
    fs::read_to_string("assets/sonnets.txt").unwrap_or_else(|_| {
        panic!(
            "File not found, please download it directly from {} and place it at assets/sonnet.txt",
            link
        )
    })
}

fn init_string() -> Vec<&'static str> {
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
            .map(|token| {
                let token = tokenizer.decode(&[*token], true).unwrap();
                let token = serde_json::to_string(&token)
                    .unwrap()
                    .trim_matches('"')
                    .to_string();
                let token: &'static str = Box::leak(token.into_boxed_str());
                token
            })
            .collect()
    } else {
        // fall back to a simple whitespace tokenizer
        log::error!("Failed to load the tokenizer, falling back to a simple whitespace tokenizer");
        contents
            .split_whitespace()
            .map(|s| {
                let s: &'static str = Box::leak(s.to_string().into_boxed_str());
                s
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_init_string_template() {
        // sanity check that the strings are the same
        let baseline = raw_string();
        // because the tokens are escaped
        let baseline = serde_json::to_string(&baseline)
            .unwrap()
            .trim_matches('"')
            .to_string();
        let template = init_string();
        let template = template.concat();
        assert_eq!(baseline, template);
    }
}
