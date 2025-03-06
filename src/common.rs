use once_cell::sync::Lazy;
use std::fs;

pub static TOKENIZED_OUTPUT: Lazy<Vec<String>> = Lazy::new(init_string);
pub static MAX_TOKENS: Lazy<usize> = Lazy::new(|| TOKENIZED_OUTPUT.len());

fn raw_string() -> String {
    fs::read_to_string("sonnet.txt").expect("Should have been able to read the file")
}

fn init_string() -> Vec<String> {
    let contents = raw_string();
    #[cfg(feature = "tokens")]
    {
        let tokenizer = tokenizers::Tokenizer::from_pretrained("meta-llama/Meta-Llama-3-8B", None)
        .expect("Should have been able to load the tokenizer");
    // splitting by whitespace would work, but this gives a better representation of tokens
    log::info!("Loaded the tokenizer");
    let tokens = tokenizer
        .encode(contents, false)
        .unwrap()
        .get_ids()
        .to_vec();
    return tokens
        .iter()
        .map(|token| tokenizer.decode(&[*token], true).unwrap())
        .collect::<Vec<String>>();
    }
    // tokenizers not found, so we split by whitespace
    #[cfg(not(feature = "tokens"))]{
    log::info!("Tokenizers not found, splitting by whitespace");
    contents.split_whitespace().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_init_string() {
        #[cfg(feature = "tokens")]
        {
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
        let contents = "This is a test";
        let tokens = contents.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        println!("{:?}", tokens);
    }
}
