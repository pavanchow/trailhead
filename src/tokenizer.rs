//! Turns raw text into a list of lowercase word tokens.

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is",
    "it", "its", "of", "on", "that", "the", "to", "was", "were", "will", "with",
];

/// Splits on any non-alphanumeric byte, lowercases, and drops empty pieces.
/// Stopword filtering is optional since short queries can lean on them.
pub fn tokenize(text: &str, filter_stopwords: bool) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .filter(|s| !filter_stopwords || !STOPWORDS.contains(s))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_punctuation_and_lowercases() {
        let tokens = tokenize("Hello, World! Rust-lang 2024.", false);
        assert_eq!(tokens, vec!["hello", "world", "rust", "lang", "2024"]);
    }

    #[test]
    fn drops_stopwords_when_asked() {
        let tokens = tokenize("the cat is on the mat", true);
        assert_eq!(tokens, vec!["cat", "mat"]);
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        let tokens = tokenize("   ---   ", false);
        assert!(tokens.is_empty());
    }
}
