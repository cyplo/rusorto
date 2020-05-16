use std::collections::HashMap;

use ::walkdir::WalkDir;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use futures::stream::{StreamExt, TryStreamExt};
use rayon::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod from;
mod walkdir;

use futures::Future;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = CommandlineOptions::from_args();

    let source_path = &options.source_path;
    let source_path = std::fs::canonicalize(source_path)
        .context(format!("Cannot read {}", source_path.to_string()))?;
    println!("processing {:?}", source_path);
    let all_files = walkdir::entries(source_path);

    let dates = all_files.map(|e| async move { e.map(|e| (e.path(), date(e.path()))) });

    let target_path = &options.target_path;
    std::fs::create_dir_all(target_path).context(format!("Cannot create {}", target_path))?;
    let target_path = std::fs::canonicalize(target_path).context(target_path.to_string())?;

    dates
        .for_each_concurrent(None, |e| async move {
            println!("{:?}", e.await);
        })
        .await;

    Ok(())
}

fn date(e: PathBuf) -> Option<(PathBuf, NaiveDateTime)> {
    if let Ok(Some(date)) = crate::from::exif::get(e.clone()) {
        return Some((e, date));
    }
    if let Some(date) = crate::from::filename::get(e.clone()) {
        return Some((e, date));
    }
    None
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "sopho")]
pub struct CommandlineOptions {
    #[structopt(help = "Directory to process", index = 1)]
    pub source_path: String,
    #[structopt(help = "Target directory to copy files to", index = 2)]
    pub target_path: String,
}
