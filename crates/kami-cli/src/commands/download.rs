//! Remote plugin download and archive extraction.
//!
//! Handles downloading `.zip` archives from URLs and extracting
//! them into the local plugins directory.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::output;

/// Downloads a `.zip` archive from `url` and extracts it into `dest_dir`.
///
/// # Errors
///
/// Returns an error on HTTP failure, invalid ZIP, or I/O issues.
pub async fn download_and_extract(url: &str, dest_dir: &Path) -> anyhow::Result<PathBuf> {
    output::print_info(&format!("Downloading {url}..."));
    let bytes = fetch_bytes(url).await?;
    output::print_info(&format!("Downloaded {} bytes", bytes.len()));
    std::fs::create_dir_all(dest_dir)?;
    extract_zip(&bytes, dest_dir)?;
    output::print_success(&format!("Extracted to {}", dest_dir.display()));
    Ok(dest_dir.to_path_buf())
}

/// Fetches raw bytes from a URL.
async fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {status} for {url}");
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("failed to read body: {e}"))?;
    Ok(bytes.to_vec())
}

/// Extracts a ZIP archive from `data` into `dest_dir`, with zip-slip prevention.
fn extract_zip(data: &[u8], dest_dir: &Path) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| anyhow::anyhow!("invalid ZIP: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| anyhow::anyhow!("ZIP entry error: {e}"))?;
        let raw_name = file
            .enclosed_name()
            .ok_or_else(|| anyhow::anyhow!("invalid path in ZIP (traversal?)"))?;
        let out_path = strip_top_dir(&raw_name, dest_dir);

        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)?;
            std::fs::write(&out_path, &buf)?;
        }
    }
    Ok(())
}

/// If `raw` has form `<dir>/x`, strip the top dir to flatten.
fn strip_top_dir(raw: &Path, dest: &Path) -> PathBuf {
    let components: Vec<_> = raw.components().collect();
    if components.len() > 1 {
        let stripped: PathBuf = components[1..].iter().collect();
        dest.join(stripped)
    } else {
        dest.join(raw)
    }
}

/// Returns `true` if the given source string looks like a URL.
pub fn is_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

/// Returns `true` if the given source looks like a GitHub shorthand
/// (`owner/repo` or `owner/repo@tag`).
pub fn is_github_shorthand(source: &str) -> bool {
    if is_url(source) {
        return false;
    }
    let base = source.split('@').next().unwrap_or(source);
    if base.contains('.') {
        return false;
    }
    let parts: Vec<&str> = base.split('/').collect();
    parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
}

/// Converts a GitHub shorthand to a release ZIP URL.
///
/// `owner/repo` → latest release assets
/// `owner/repo@v1.0.0` → tagged release
pub fn github_release_url(shorthand: &str) -> String {
    let (path, tag) = match shorthand.split_once('@') {
        Some((p, t)) => (p, t.to_string()),
        None => (shorthand, "latest".to_string()),
    };

    if tag == "latest" {
        format!("https://github.com/{path}/releases/latest/download/plugin.zip")
    } else {
        format!("https://github.com/{path}/releases/download/{tag}/plugin.zip")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_url_detects_http() {
        assert!(is_url("https://example.com/plugin.zip"));
        assert!(is_url("http://localhost:8080/test.zip"));
        assert!(!is_url("./local/path"));
        assert!(!is_url("owner/repo"));
    }
    #[test]
    fn github_shorthand_patterns() {
        assert!(is_github_shorthand("owner/repo"));
        assert!(is_github_shorthand("kami-tools/fetch@v1.0.0"));
        assert!(!is_github_shorthand("https://example.com"));
        assert!(!is_github_shorthand("./local/path"));
        assert!(!is_github_shorthand("org/repo/extra")); // 3 parts → false
    }
    #[test]
    fn github_url_generation() {
        let latest = github_release_url("kami/fetch");
        assert!(latest.contains("latest/download"));
        let tagged = github_release_url("kami/fetch@v1.2.0");
        assert!(tagged.contains("v1.2.0/plugin.zip"));
    }
    #[test]
    fn strip_flattens_nested() {
        let raw = Path::new("my-plugin-v1/tool.toml");
        let dest = Path::new("/tmp/out");
        assert_eq!(strip_top_dir(raw, dest), dest.join("tool.toml"));
    }
    #[test]
    fn strip_single_component_keeps_as_is() {
        let raw = Path::new("tool.toml");
        let dest = Path::new("/tmp/out");
        assert_eq!(strip_top_dir(raw, dest), dest.join("tool.toml"));
    }
}
