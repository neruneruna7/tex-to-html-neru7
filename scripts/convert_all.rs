#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! ```

use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() -> Result<()> {
    let sources_dir = PathBuf::from("sources");

    if !sources_dir.is_dir() {
        bail!("sources directory not found: {}", sources_dir.display());
    }

    let archives = collect_archives(&sources_dir)?;

    if archives.is_empty() {
        bail!("no .tar.gz files found under {}", sources_dir.display());
    }

    let mut failed = Vec::new();

    for archive in archives {
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
            println!("OK: {}", archive.display());
        } else {
            eprintln!("FAILED: {}", archive.display());
            failed.push(archive);
        }
    }

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
