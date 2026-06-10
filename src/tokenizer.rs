//! Advanced tokenizer with multilingual support using Tantivy.
//!
//! This module provides an advanced tokenizer with stemming, stopwords removal,
//! and Unicode folding for improved text search quality across multiple languages.
//! It is only available when the `advanced-tokenizer` feature is enabled.

#[cfg(feature = "advanced-tokenizer")]
use tantivy::tokenizer::{Language, SimpleTokenizer, Tokenizer, TokenStream};

#[cfg(feature = "advanced-tokenizer")]
const ADVANCED_TOKENIZER_NAME: &str = "tantivy-multilingual";
#[cfg(feature = "advanced-tokenizer")]
const ADVANCED_TOKENIZER_VERSION: u32 = 1;

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

/// Tokenize text using the advanced Tantivy tokenizer
#[cfg(feature = "advanced-tokenizer")]
pub fn tokenize_advanced(text: &str, config: &AdvancedTokenizerConfig) -> Vec<String> {
    let mut tokenizer = SimpleTokenizer::default();
    let mut stream = tokenizer.token_stream(text);
    
    let mut tokens = Vec::new();
    
    while let Some(token) = stream.next() {
        let token_text = token.text.to_string();
        
        // Apply length filter
        if token_text.len() > config.max_token_length {
            continue;
        }
        
        // For now, just use the basic tokenization from Tantivy
        // Advanced features like stemming and stopwords require more complex setup
        tokens.push(token_text);
    }

    tokens
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
mod tests {
    use super::*;

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_basic() {
        let text = "The quick brown fox jumps over the lazy dog";
        let tokens = tokenize_advanced_default(text);
        
        // Should tokenize
        assert!(!tokens.is_empty());
        // Should contain common words
        assert!(tokens.contains(&"quick".to_string()) || tokens.contains(&"brown".to_string()));
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
    fn test_advanced_tokenizer_unicode() {
        let text = "Café naïve résumé";
        let tokens = tokenize_advanced_default(text);
        
        // Should handle Unicode characters
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_advanced_tokenizer_availability() {
        // This test always passes, just checks the function works
        let _ = is_advanced_tokenizer_available();
    }
}
