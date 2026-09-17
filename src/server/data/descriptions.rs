//! Load generated stellar-host prose from the local content directory.
use std::{collections::HashMap, fs, path::Path};

use pulldown_cmark::{Options, Parser, html::push_html};

/// Read one `description.md` per `<content-dir>/<system-id>/` subdirectory
/// and render it to HTML keyed by system identifier. Subdirectories without
/// a `description.md` are skipped; a missing content directory yields an
/// empty map.
pub fn load_host_descriptions(content_dir: &Path) -> HashMap<String, String> {
    let mut descriptions = HashMap::new();
    let entries = match fs::read_dir(content_dir) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::info!(
                "Stellar-host descriptions unavailable at {}: {}",
                content_dir.display(),
                error
            );
            return descriptions;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path().join("description.md");
        if !path.is_file() {
            continue;
        }
        let Ok(markdown) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        descriptions.insert(id, render_description(&markdown));
    }
    descriptions
}

/// Render an article to HTML. Raw HTML characters are escaped before
/// parsing; the article's own level-one title is kept and rendered.
pub fn render_description(markdown: &str) -> String {
    let body = markdown
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&body, options);
    let mut html = String::new();
    push_html(&mut html, parser);
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_title_and_paragraphs_with_escaping() {
        let html = render_description(
            "# TRAPPIST-1 and its planets\n\nFirst paragraph.\n\nSecond \
             <paragraph>.",
        );
        assert!(html.contains("<h1>TRAPPIST-1 and its planets</h1>"));
        assert!(html.contains("<p>First paragraph.</p>"));
        assert!(html.contains("&lt;paragraph&gt;"));
    }

    #[test]
    fn keeps_body_when_no_title_is_present() {
        let html = render_description("Only paragraph.");
        assert!(html.contains("Only paragraph."));
    }

    #[test]
    fn loads_only_directories_with_descriptions() {
        let directory = std::env::temp_dir()
            .join(format!("exodata-descriptions-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("lhs-1140")).unwrap();
        fs::create_dir_all(directory.join("empty-system")).unwrap();
        fs::write(
            directory.join("lhs-1140/description.md"),
            "# LHS 1140\n\nProse.",
        )
        .unwrap();
        let descriptions = load_host_descriptions(&directory);
        assert_eq!(descriptions.len(), 1);
        assert!(descriptions["lhs-1140"].contains("Prose."));
        let missing = load_host_descriptions(&directory.join("absent"));
        assert!(missing.is_empty());
        fs::remove_dir_all(&directory).unwrap();
    }
}
