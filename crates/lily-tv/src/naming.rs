//! File names and extensions for exported channels, groups and sources.

pub const CHANNEL_EXTENSION: &str = "otv";
pub const GROUP_EXTENSION: &str = "otvg";
pub const SOURCE_EXTENSION: &str = "otvp";
pub const FAVORITES_EXTENSION: &str = "otvf";
pub const RECORD_EXTENSION: &str = "mp4";

/// Makes `name` safe to use as a file name.
pub fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if r#"/\:*?"<>|"#.contains(c) { '_' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    if cleaned.is_empty() {
        "untitled".to_string()
    } else {
        cleaned.to_string()
    }
}

/// Timestamp for file names, e.g. `2026-11-07-18-30-05`.
pub fn date_stamp() -> String {
    chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}

/// The extension a downloaded stream should get, guessed from its URL.
pub fn url_extension(url: &str) -> &str {
    match url.rsplit_once('.') {
        Some((_, ext)) if !ext.starts_with("php?") && !ext.contains('/') && !ext.is_empty() => ext,
        _ => "mp4",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_reserved_characters() {
        assert_eq!(sanitize_file_name("Sky Sports | F1: HD?"), "Sky Sports _ F1_ HD_");
        assert_eq!(sanitize_file_name("..."), "untitled");
        assert_eq!(sanitize_file_name("  .hidden. "), "hidden");
    }

    #[test]
    fn guesses_extension_from_url() {
        assert_eq!(url_extension("http://x/movie/u/p/123.mkv"), "mkv");
        assert_eq!(url_extension("http://x/get.php?id=1"), "mp4");
        assert_eq!(url_extension("http://example.com/stream"), "mp4");
    }
}
