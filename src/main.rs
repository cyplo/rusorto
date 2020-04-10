use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use structopt::StructOpt;
use ::walkdir::WalkDir;
use futures::stream::{StreamExt, TryStreamExt};
use std::path::PathBuf;

mod walkdir;
mod from;

use tokio::prelude::*;
use futures::Future;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = CommandlineOptions::from_args();

    let dir = &options.dir;
    let path = std::fs::canonicalize(dir).context(dir.to_string())?;
    println!("processing {:?}", path);
    let entries = walkdir::entries(path);

    let dates = entries.filter_map(|e| async move {
        e.map(|e| date(e.path())).ok()
    });

    dates.for_each_concurrent(None, |e| async move {
        println!("{:?}", e);
    }).await;

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
