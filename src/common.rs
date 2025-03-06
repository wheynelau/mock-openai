use once_cell::sync::Lazy;
use std::fs;

pub static TOKENIZED_OUTPUT: Lazy<Vec<String>> = Lazy::new(init_string);
pub static MAX_TOKENS: Lazy<usize> = Lazy::new(|| TOKENIZED_OUTPUT.len());

fn raw_string() -> String {
    // load random.txt
    fs::read_to_string("random.txt").expect("Should have been able to read the file")
}

fn init_string() -> Vec<String> {
    // load random.txt
    let contents = raw_string();
    let tokenizer = tokenizers::Tokenizer::from_pretrained("meta-llama/Meta-Llama-3-8B", None)
        .expect("Should have been able to load the tokenizer");
    // splitting by whitespace would work, but this gives a better representation of tokens
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
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_init_string() {
        let tokenizer = tokenizers::Tokenizer::from_pretrained("meta-llama/Meta-Llama-3-8B", None)
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
