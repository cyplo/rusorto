use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use futures::stream::{StreamExt, TryStreamExt};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use anyhow::*;
mod from;
mod walkdir;

use futures::Future;
use futures::FutureExt;
use std::fs;
use std::sync::Arc;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = CommandlineOptions::from_args();

    let source_path = &options.source_path;
    let source_path = fs::canonicalize(source_path)
        .context(format!("Cannot read {}", source_path.to_string()))?;
    let source_path = Arc::new(source_path);
    println!("processing {:?}", source_path);
    let all_files = walkdir::entries(source_path.as_ref());

    let dates = all_files.map(|e| async { e.map(|e| (e.path(), date(e.path()))) });

    let target_path = &options.target_path;
    fs::create_dir_all(target_path).context(format!("Cannot create {}", target_path))?;
    let target_path = fs::canonicalize(target_path).context(target_path.to_string())?;

    let target_paths =
        dates.map(|d| d.then(|f| async { f.map(|pd| target_path_for(source_path.as_ref(), pd)) }));

    target_paths
        .for_each_concurrent(None, |e| async move {
            let e = e.await;
            println!("{:?}", e);
        })
        .await;

    Ok(())
}

fn target_path_for(
    source_path: impl Into<PathBuf>,
    path_and_date: (PathBuf, Option<NaiveDateTime>),
) -> Result<(PathBuf, PathBuf, Option<NaiveDateTime>), Error> {
    let path = path_and_date.0;
    let date = path_and_date.1;
    let target = path
        .strip_prefix(source_path.into())
        .map(|p| p.to_string_lossy())
        .map_err(|e| anyhow!(e));
    if let Err(e) = target {
        return Err(anyhow!(e));
    }
    let target = target.unwrap().to_string();
    let target = Path::new(&target);
    Ok((path, target.to_path_buf(), date))
}

fn date(e: PathBuf) -> Option<NaiveDateTime> {
    if let Ok(Some(date)) = crate::from::exif::get(e.clone()) {
        return Some(date);
    }
    if let Some(date) = crate::from::filename::get(e.clone()) {
        return Some(date);
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
