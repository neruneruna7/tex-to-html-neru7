#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! ```

use anyhow::{Context, Result};
use std::{fs, path::Path};

fn main() -> Result<()> {
    remove_dir_if_exists("work")?;
    remove_dir_if_exists("dist")?;

    println!("Removed: work/, dist/");

    Ok(())
}

fn remove_dir_if_exists(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }

    Ok(())
}
