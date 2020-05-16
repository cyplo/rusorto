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
    let target_path = Arc::new(target_path);
    let target_paths = dates.map(|d| {
        d.then(|f| async {
            f.map(|date| target_path_for(source_path.as_ref(), target_path.as_ref(), date))
        })
    });

    // TODO - make sure there are no collisions - detect duplicates in target paths
    // for duplicates - check hash if true duplicate
    // check free space on disk before copy

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
    target_path: impl Into<PathBuf>,
    path_and_date: (PathBuf, Option<NaiveDateTime>),
) -> Result<(PathBuf, PathBuf), Error> {
    let path = path_and_date.0;
    let date = path_and_date.1;
    let source_path = source_path.into();
    let new_name_from_date: Result<String> = match date {
        Some(date) => {
            let date_text = date.format("%Y%m%d_%H%M%S").to_string();
            let extension = path.extension().map_or("".to_string(), |e| {
                format!(".{}", e.to_string_lossy().to_lowercase())
            });
            Ok(format!("{}{}", date_text, extension))
        }
        None => {
            let target = path.strip_prefix(&source_path).map_err(|e| anyhow!(e));
            match target {
                Ok(target) => Ok(target.to_string_lossy().to_string()),
                Err(e) => Err(anyhow!(e)),
            }
        }
    };
    let target = &target_path.into().join(new_name_from_date?);
    let target = Path::new(&target);
    Ok((path, target.to_path_buf()))
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
