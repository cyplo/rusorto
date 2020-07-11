use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::*;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use futures::stream::StreamExt;
use futures::FutureExt;
use structopt::StructOpt;

mod from;
mod walkdir;

static UNSORTED_PATH_SEGMENT: &str = "unsorted";

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
            f.map(|path_and_date| {
                (
                    path_and_date.0.to_owned(),
                    target_path_for(source_path.as_ref(), target_path.as_ref(), path_and_date),
                )
            })
        })
    });

    let unsorted_directory = target_path.join(UNSORTED_PATH_SEGMENT);
    fs::create_dir_all(&unsorted_directory)
        .context(format!("Cannot create {:?}", &unsorted_directory))?;

    // TODO - make sure there are no collisions - detect duplicates in target paths
    // for duplicates - check hash if true duplicate
    // check free space on disk before copy

    target_paths
        .for_each_concurrent(None, |e| async move {
            let to_and_from = e.await;
            match to_and_from {
                Ok(to_and_from) => {
                    let from = to_and_from.0;
                    let to = to_and_from.1;
                    match to {
                        Ok(to) => copy(from, to),
                        Err(e) => eprintln!("{}", e),
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        })
        .await;

    // TODO: delete unsorted if empty

    Ok(())
}

fn copy(from: PathBuf, to: NewPath) {
    match to {
        NewPath::Simple(path) => {
            println!("copying {:?} to {:?}", &from, &path);
        }
        NewPath::UnderNewDirectory(path) => {
            println!("copying {:?} to {:?} (new parent)", &from, &path);
        }
    }
}

#[derive(Debug)]
enum NewPath {
    Simple(PathBuf),
    UnderNewDirectory(PathBuf),
}

fn target_path_for(
    source_path: impl Into<PathBuf>,
    target_path: impl Into<PathBuf>,
    path_and_date: (PathBuf, Option<NaiveDateTime>),
) -> Result<NewPath> {
    let path = path_and_date.0;
    let date = path_and_date.1;
    let source_path = source_path.into();
    match date {
        Some(date) => {
            let file_name = file_name_from_date(&path, date);
            let full_path = &target_path.into().join(file_name);
            Ok(NewPath::Simple(full_path.to_owned()))
        }
        None => {
            let target = path.strip_prefix(&source_path).map_err(|e| anyhow!(e));
            match target {
                Ok(target) => {
                    let parent = target_path.into().join(UNSORTED_PATH_SEGMENT);
                    let full_path = parent.join(target);
                    Ok(NewPath::UnderNewDirectory(full_path))
                }
                Err(e) => Err(anyhow!(e)),
            }
        }
    }
}

fn file_name_from_date(path: &PathBuf, date: NaiveDateTime) -> String {
    let date_text = date.format("%Y%m%d_%H%M%S").to_string();
    let extension = path.extension().map_or("".to_string(), |e| {
        format!(".{}", e.to_string_lossy().to_lowercase())
    });
    format!("{}{}", date_text, extension)
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
