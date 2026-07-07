//! Transcript file I/O and processing.
//!
//! Handles reading and writing transcript text files from disk.
//! When the `async-io` feature is enabled, file operations use
//! `tokio::fs`; in-memory processing always remains synchronous.

use crate::error::Result;

#[cfg(not(feature = "async-io"))]
use std::fs;

#[cfg(feature = "async-io")]
use tokio::fs;

/// Read transcript content from a file path.
#[cfg(not(feature = "async-io"))]
pub fn read_transcript(path: impl AsRef<std::path::Path>) -> Result<String> {
    let data = fs::read(path.as_ref())?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

/// Read transcript content from a file path (async).
#[cfg(feature = "async-io")]
pub async fn read_transcript(path: impl AsRef<std::path::Path>) -> Result<String> {
    let data = fs::read(path.as_ref()).await?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

/// Write transcript content to a file path.
#[cfg(not(feature = "async-io"))]
pub fn write_transcript(path: impl AsRef<std::path::Path>, content: &str) -> Result<()> {
    fs::write(path.as_ref(), content)?;
    Ok(())
}

/// Write transcript content to a file path (async).
#[cfg(feature = "async-io")]
pub async fn write_transcript(path: impl AsRef<std::path::Path>, content: &str) -> Result<()> {
    fs::write(path.as_ref(), content).await?;
    Ok(())
}

/// Open a transcript file for append operations.
#[cfg(not(feature = "async-io"))]
pub fn append_transcript(path: impl AsRef<std::path::Path>, line: &str) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Open a transcript file for append operations (async).
#[cfg(feature = "async-io")]
pub async fn append_transcript(path: impl AsRef<std::path::Path>, line: &str) -> Result<()> {
    use tokio::io::AsyncWriteExt as _;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .await?;
    file.write_all(format!("{line}\n").as_bytes()).await?;
    Ok(())
}

/// Split transcript text into non-empty line segments (always synchronous).
pub fn segment_transcript(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Count words in transcript text (always synchronous).
pub fn word_count(content: &str) -> usize {
    content.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_transcript() {
        let text = "hello world\n\nfoo bar\n  ";
        let segments = segment_transcript(text);
        assert_eq!(segments, &["hello world", "foo bar"]);
    }

    #[test]
    fn test_word_count() {
        assert_eq!(word_count("hello world foo"), 3);
        assert_eq!(word_count(""), 0);
    }
}
