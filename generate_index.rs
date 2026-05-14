#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! html-escape = "0.2"
//! ```

use anyhow::{Context, Result};
use html_escape::encode_text;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let dist_dir = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("dist"),
    };

    if !dist_dir.is_dir() {
        anyhow::bail!("dist directory not found: {}", dist_dir.display());
    }

    let entries = collect_document_dirs(&dist_dir)?;

    let html = render_index(&entries);

    let index_path = dist_dir.join("index.html");
    fs::write(&index_path, html)
        .with_context(|| format!("failed to write {}", index_path.display()))?;

    println!("Generated: {}", index_path.display());

    Ok(())
}

fn collect_document_dirs(dist_dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();

    for entry in
        fs::read_dir(dist_dir).with_context(|| format!("failed to read {}", dist_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        // dist/<name>/index.html があるものだけトップページに載せる
        if path.join("index.html").is_file() {
            names.push(name.to_string());
        }
    }

    names.sort();

    Ok(names)
}

fn render_index(names: &[String]) -> String {
    let mut html = String::new();

    html.push_str(
        r#"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Converted TeX Documents</title>
</head>
<body>
  <h1>Converted TeX Documents</h1>
  <ul>
"#,
    );

    for name in names {
        let escaped_name = encode_text(name);
        html.push_str(&format!(
            r#"    <li><a href="./{}/">{}</a></li>
"#,
            escaped_name, escaped_name
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
