//! Advanced tokenizer with multilingual support using Tantivy.
//!
//! This module provides an advanced tokenizer with stemming, stopwords removal,
//! and Unicode folding for improved text search quality across multiple languages.
//! It is only available when the `advanced-tokenizer` feature is enabled.

#[cfg(feature = "advanced-tokenizer")]
use tantivy::tokenizer::{
    AsciiFoldingFilter, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer, StopWordFilter,
    TokenStream,
};
/// Language used for stemming and stopwords. Re-exported from Tantivy so users
/// can name it without depending on Tantivy directly.
#[cfg(feature = "advanced-tokenizer")]
pub use tantivy::tokenizer::{Language, TextAnalyzer};
/// Advanced tokenizer configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedTokenizerConfig {
    /// Language for stemming and stopwords
    pub language: Language,
    /// Maximum token length (tokens longer than this are filtered out)
    pub max_token_length: usize,
    /// Whether to remove stopwords
    pub remove_stopwords: bool,
    /// Whether to apply stemming
    pub apply_stemming: bool,
}

impl Default for AdvancedTokenizerConfig {
    fn default() -> Self {
        Self {
            language: Language::English,
            max_token_length: 40,
            remove_stopwords: true,
            apply_stemming: true,
        }
    }
}

/// Build a reusable advanced Tantivy analyzer for the given configuration.
///
/// The analyzer holds the stemming/stopwords pipeline; building it is the
/// expensive part, so construct it once per batch/call and reuse it via
/// [`tokenize_with_analyzer`] instead of rebuilding per text.
// Source: https://docs.rs/tantivy/0.26.1/tantivy/tokenizer/struct.TextAnalyzer.html#method.token_stream
#[cfg(feature = "advanced-tokenizer")]
pub fn build_advanced_analyzer(config: &AdvancedTokenizerConfig) -> TextAnalyzer {
    if config.apply_stemming && config.remove_stopwords {
        // Both stemming and stopwords
        if let Some(stopword_filter) = StopWordFilter::new(config.language) {
            TextAnalyzer::builder(SimpleTokenizer::default())
                .filter(RemoveLongFilter::limit(config.max_token_length))
                .filter(LowerCaser)
                .filter(AsciiFoldingFilter)
                .filter(Stemmer::new(config.language))
                .filter(stopword_filter)
                .build()
        } else {
            // Stemming only (stopwords not available for this language)
            TextAnalyzer::builder(SimpleTokenizer::default())
                .filter(RemoveLongFilter::limit(config.max_token_length))
                .filter(LowerCaser)
                .filter(AsciiFoldingFilter)
                .filter(Stemmer::new(config.language))
                .build()
        }
    } else if config.apply_stemming {
        // Stemming only
        TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(config.max_token_length))
            .filter(LowerCaser)
            .filter(AsciiFoldingFilter)
            .filter(Stemmer::new(config.language))
            .build()
    } else if config.remove_stopwords {
        // Stopwords only
        if let Some(stopword_filter) = StopWordFilter::new(config.language) {
            TextAnalyzer::builder(SimpleTokenizer::default())
                .filter(RemoveLongFilter::limit(config.max_token_length))
                .filter(LowerCaser)
                .filter(AsciiFoldingFilter)
                .filter(stopword_filter)
                .build()
        } else {
            // No stopwords available, use basic tokenizer
            TextAnalyzer::builder(SimpleTokenizer::default())
                .filter(RemoveLongFilter::limit(config.max_token_length))
                .filter(LowerCaser)
                .filter(AsciiFoldingFilter)
                .build()
        }
    } else {
        // Basic tokenizer without stemming or stopwords
        TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(config.max_token_length))
            .filter(LowerCaser)
            .filter(AsciiFoldingFilter)
            .build()
    }
}

/// Tokenize text with a prebuilt analyzer, reusing it across calls.
///
/// `token_stream` borrows the analyzer only for the lifetime of the stream, so
/// the same analyzer can be reused sequentially for many texts.
#[cfg(feature = "advanced-tokenizer")]
pub fn tokenize_with_analyzer(analyzer: &mut TextAnalyzer, text: &str) -> Vec<String> {
    let mut stream = analyzer.token_stream(text);

    let mut tokens = Vec::new();

    while let Some(token) = stream.next() {
        tokens.push(token.text.to_string());
    }

    tokens
}

/// Tokenize text using the advanced Tantivy tokenizer
#[cfg(feature = "advanced-tokenizer")]
pub fn tokenize_advanced(text: &str, config: &AdvancedTokenizerConfig) -> Vec<String> {
    let mut analyzer = build_advanced_analyzer(config);
    tokenize_with_analyzer(&mut analyzer, text)
}

/// Tokenize text with default configuration
#[cfg(feature = "advanced-tokenizer")]
pub fn tokenize_advanced_default(text: &str) -> Vec<String> {
    tokenize_advanced(text, &AdvancedTokenizerConfig::default())
}

/// Check if advanced tokenizer is available
pub fn is_advanced_tokenizer_available() -> bool {
    cfg!(feature = "advanced-tokenizer")
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_basic() {
        let text = "The quick brown fox jumps over the lazy dog";
        let tokens = tokenize_advanced_default(text);

        // Should tokenize
        assert!(!tokens.is_empty());
        // Should contain common words (may be stemmed)
        assert!(tokens
            .iter()
            .any(|t| t.contains("quick") || t.contains("brown")));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_stemming() {
        let config = AdvancedTokenizerConfig {
            apply_stemming: true,
            remove_stopwords: false,
            ..Default::default()
        };

        let text = "The jumping fox runs quickly";
        let tokens = tokenize_advanced(text, &config);

        // "jumping" should be stemmed to "jump"
        assert!(tokens.iter().any(|t| t == "jump" || t.contains("jump")));
        // "quickly" should be stemmed to "quickli" or similar
        assert!(tokens.iter().any(|t| t.contains("quick")));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_stopwords() {
        let config_with_stopwords = AdvancedTokenizerConfig {
            remove_stopwords: true,
            apply_stemming: false,
            ..Default::default()
        };

        let config_without_stopwords = AdvancedTokenizerConfig {
            remove_stopwords: false,
            apply_stemming: false,
            ..Default::default()
        };

        let text = "The quick brown fox jumps over the lazy dog";
        let tokens_with = tokenize_advanced(text, &config_with_stopwords);
        let tokens_without = tokenize_advanced(text, &config_without_stopwords);

        // With stopwords removed, should have fewer tokens
        assert!(tokens_with.len() < tokens_without.len());

        // Common stopwords like "the", "and", "is" should be removed
        assert!(!tokens_with.iter().any(|t| t == "the"));
        assert!(!tokens_with.iter().any(|t| t == "is"));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_multilingual_spanish() {
        let config = AdvancedTokenizerConfig {
            language: Language::Spanish,
            ..Default::default()
        };

        let text = "El perro rápido salta sobre el perro perezoso";
        let tokens = tokenize_advanced(text, &config);

        assert!(!tokens.is_empty());
        // Spanish stopwords like "el", "sobre" should be removed
        assert!(!tokens.iter().any(|t| t == "el"));
        assert!(!tokens.iter().any(|t| t == "sobre"));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_multilingual_french() {
        let config = AdvancedTokenizerConfig {
            language: Language::French,
            ..Default::default()
        };

        let text = "Le chien rapide saute par-dessus le chien paresseux";
        let tokens = tokenize_advanced(text, &config);

        assert!(!tokens.is_empty());
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_multilingual_german() {
        let config = AdvancedTokenizerConfig {
            language: Language::German,
            ..Default::default()
        };

        let text = "Der schnelle Hund springt über den faulen Hund";
        let tokens = tokenize_advanced(text, &config);

        assert!(!tokens.is_empty());
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_length_filter() {
        let config = AdvancedTokenizerConfig {
            max_token_length: 5,
            ..Default::default()
        };

        let text = "The quick brown fox jumps over the lazy dog";
        let tokens = tokenize_advanced(text, &config);

        // All tokens should be <= 5 characters
        for token in &tokens {
            assert!(token.len() <= 5);
        }
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_empty_text() {
        let text = "";
        let tokens = tokenize_advanced_default(text);

        assert!(tokens.is_empty());
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_unicode_folding() {
        let text = "Café naïve résumé";
        let tokens = tokenize_advanced_default(text);

        // Should handle Unicode characters and fold them to ASCII
        assert!(!tokens.is_empty());
        // "café" should become "cafe" or similar
        assert!(tokens.iter().any(|t| t.contains("cafe")));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_combined_features() {
        let config = AdvancedTokenizerConfig {
            apply_stemming: true,
            remove_stopwords: true,
            ..Default::default()
        };

        let text = "The jumping fox runs quickly and the lazy dog";
        let tokens = tokenize_advanced(text, &config);

        // Should have fewer tokens due to stopwords removal
        assert!(!tokens.is_empty());
        // Should not contain stopwords
        assert!(!tokens.iter().any(|t| t == "the"));
        assert!(!tokens.iter().any(|t| t == "and"));
        // Should contain stemmed words
        assert!(tokens.iter().any(|t| t.contains("jump")));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_analyzer_reuse_matches_fresh_build() {
        let config = AdvancedTokenizerConfig::default();
        let mut analyzer = build_advanced_analyzer(&config);
        let texts = [
            "The jumping fox runs quickly",
            "Café naïve résumé",
            "",
            "another batch item",
        ];
        for text in texts {
            let reused = tokenize_with_analyzer(&mut analyzer, text);
            let fresh = tokenize_advanced(text, &config);
            assert_eq!(reused, fresh);
        }
    }

    #[test]
    fn test_advanced_tokenizer_availability() {
        // This test always passes, just checks the function works
        let _ = is_advanced_tokenizer_available();
    }
}
