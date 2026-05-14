#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! html-escape = "0.2"
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! toml = "0.8"
//! ```

use anyhow::{Context, Result};
use html_escape::{decode_html_entities, encode_double_quoted_attribute, encode_text};
use regex::Regex;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

const TITLE_MAPPING_FILE: &str = "DistTitles.toml";

#[derive(Debug)]
struct DocumentEntry {
    dir_name: String,
    title: String,
    title_source: TitleSource,
}

#[derive(Debug)]
enum TitleSource {
    Config,
    HtmlTitle,
    Fallback,
}

#[derive(Debug, Deserialize, Default)]
struct TitleConfig {
    #[serde(default)]
    titles: HashMap<String, String>,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let dist_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist"));

    let config_path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TITLE_MAPPING_FILE));

    if !dist_dir.is_dir() {
        anyhow::bail!("dist directory not found: {}", dist_dir.display());
    }

    let config = load_title_config(&config_path)?;

    let entries = collect_document_entries(&dist_dir, &config)?;

    let html = render_index(&entries);

    let index_path = dist_dir.join("index.html");
    fs::write(&index_path, html)
        .with_context(|| format!("failed to write {}", index_path.display()))?;

    println!("Generated: {}", index_path.display());

    Ok(())
}

fn load_title_config(path: &Path) -> Result<TitleConfig> {
    if !path.exists() {
        return Ok(TitleConfig::default());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    let config: TitleConfig =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(config)
}

fn collect_document_entries(dist_dir: &Path, config: &TitleConfig) -> Result<Vec<DocumentEntry>> {
    let mut entries = Vec::new();

    for entry in
        fs::read_dir(dist_dir).with_context(|| format!("failed to read {}", dist_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        let index_path = path.join("index.html");

        if !index_path.is_file() {
            continue;
        }

        let (title, title_source) = resolve_title(dir_name, &index_path, config);

        entries.push(DocumentEntry {
            dir_name: dir_name.to_string(),
            title,
            title_source,
        });
    }

    entries.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));

    Ok(entries)
}

fn resolve_title(dir_name: &str, index_path: &Path, config: &TitleConfig) -> (String, TitleSource) {
    if let Some(title) = config.titles.get(dir_name) {
        let title = title.trim();

        if !title.is_empty() {
            return (title.to_string(), TitleSource::Config);
        }
    }

    if let Ok(title) = extract_html_title(index_path) {
        return (title, TitleSource::HtmlTitle);
    }

    (dir_name.to_string(), TitleSource::Fallback)
}

fn extract_html_title(index_path: &Path) -> Result<String> {
    let html = fs::read_to_string(index_path)
        .with_context(|| format!("failed to read {}", index_path.display()))?;

    let re = Regex::new(r"(?is)<title[^>]*>(.*?)</title>")?;

    let Some(caps) = re.captures(&html) else {
        anyhow::bail!("title element not found: {}", index_path.display());
    };

    let raw_title = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();

    let title = decode_html_entities(raw_title)
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if title.is_empty() {
        anyhow::bail!("empty title: {}", index_path.display());
    }

    Ok(title)
}

fn render_index(entries: &[DocumentEntry]) -> String {
    let mut html = String::new();

    html.push_str(
        r#"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Converted TeX Documents</title>
  <style>
    :root {
      color-scheme: light;
    }
    body {
      background: white;
      color: black;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      max-width: 960px;
      margin: 3rem auto;
      padding: 0 1rem;
      line-height: 1.6;
    }
    h1 {
      margin-bottom: 1.5rem;
    }
    ul {
      padding-left: 1.5rem;
    }
    li {
      margin: 0.5rem 0;
    }
    a {
      color: #0645ad;
    }
    .id {
      color: #666;
      font-size: 0.9em;
      margin-left: 0.4rem;
    }
    .source {
      color: #999;
      font-size: 0.8em;
      margin-left: 0.4rem;
    }
  </style>
</head>
<body>
  <h1>Converted TeX Documents</h1>
  <ul>
"#,
    );

    for entry in entries {
        let href = format!("./{}/", entry.dir_name);

        let escaped_href = encode_double_quoted_attribute(&href);
        let escaped_title = encode_text(&entry.title);
        let escaped_dir_name = encode_text(&entry.dir_name);
        let source = match entry.title_source {
            TitleSource::Config => "config",
            TitleSource::HtmlTitle => "html",
            TitleSource::Fallback => "fallback",
        };

        html.push_str(&format!(
            r#"    <li><a href="{escaped_href}">{escaped_title}</a><span class="id">({escaped_dir_name})</span><span class="source">[{source}]</span></li>
"#
        ));
    }

    html.push_str(
        r#"  </ul>
</body>
</html>
"#,
    );

    html
}
