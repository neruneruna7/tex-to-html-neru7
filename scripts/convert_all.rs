#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! sha2 = "0.10"
//! hex = "0.4"
//! ```

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() -> Result<()> {
    let sources_dir = PathBuf::from("sources");
    let dist_dir = PathBuf::from("dist");
    let cache_dir = PathBuf::from(".tex-cache");

    if !sources_dir.is_dir() {
        bail!("sources directory not found: {}", sources_dir.display());
    }

    fs::create_dir_all(&dist_dir)
        .with_context(|| format!("failed to create {}", dist_dir.display()))?;
    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;

    let archives = collect_archives(&sources_dir)?;

    if archives.is_empty() {
        bail!("no .tar.gz files found under {}", sources_dir.display());
    }

    let mut failed = Vec::new();
    let mut converted = 0usize;
    let mut skipped = 0usize;

    for archive in archives {
        let base_name = archive_base_name(&archive)?;
        let digest = sha256_file(&archive)?;

        let stamp_path = cache_dir.join(format!("{base_name}.sha256"));
        let output_index = dist_dir.join(&base_name).join("index.html");

        if is_already_converted(&stamp_path, &digest, &output_index)? {
            println!("Skip unchanged: {}", archive.display());
            skipped += 1;
            continue;
        }

        println!("========================================");
        println!("Converting: {}", archive.display());
        println!("========================================");

        let status = Command::new("rust-script")
            .arg("scripts/convert_tex.rs")
            .arg(&archive)
            .stdin(Stdio::null())
            .status()
            .with_context(|| format!("failed to execute rust-script for {}", archive.display()))?;

        if status.success() {
            fs::write(&stamp_path, &digest)
                .with_context(|| format!("failed to write {}", stamp_path.display()))?;

            println!("OK: {}", archive.display());
            converted += 1;
        } else {
            eprintln!("FAILED: {}", archive.display());
            failed.push(archive);
        }
    }

    println!();
    println!("Summary:");
    println!("  converted: {converted}");
    println!("  skipped:   {skipped}");
    println!("  failed:    {}", failed.len());

    if !failed.is_empty() {
        eprintln!();
        eprintln!("Failed archives:");
        for archive in failed {
            eprintln!("  {}", archive.display());
        }

        bail!("some archives failed to convert");
    }

    Ok(())
}

fn collect_archives(sources_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut archives = Vec::new();

    for entry in fs::read_dir(sources_dir)
        .with_context(|| format!("failed to read {}", sources_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_tar_gz(&path) {
            archives.push(path);
        }
    }

    archives.sort();

    Ok(archives)
}

fn is_tar_gz(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|name| name.ends_with(".tar.gz"))
        .unwrap_or(false)
}

fn archive_base_name(path: &Path) -> Result<String> {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("invalid archive file name: {}", path.display()))?;

    let base_name = file_name
        .strip_suffix(".tar.gz")
        .with_context(|| format!("archive must end with .tar.gz: {}", path.display()))?;

    if base_name.is_empty() {
        bail!("empty archive base name: {}", path.display());
    }

    Ok(base_name.to_string())
}

fn sha256_file(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut reader = BufReader::new(file);

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024 * 64];

    loop {
        let n = reader
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn is_already_converted(stamp_path: &Path, digest: &str, output_index: &Path) -> Result<bool> {
    if !output_index.is_file() {
        return Ok(false);
    }

    if !stamp_path.is_file() {
        return Ok(false);
    }

    let previous_digest = fs::read_to_string(stamp_path)
        .with_context(|| format!("failed to read {}", stamp_path.display()))?;

    Ok(previous_digest.trim() == digest)
}
