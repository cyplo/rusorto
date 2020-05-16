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

    let dir = &options.dir;
    let source_path = std::fs::canonicalize(dir).context(dir.to_string())?;
    println!("processing {:?}", source_path);
    let all_files = walkdir::entries(source_path);

    let dates = all_files.map(|e| async move { e.map(|e| date(e.path())) });

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
    pub dir: String,
}
