use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use structopt::StructOpt;
use walkdir::WalkDir;
use std::fs;

fn main() -> Result<()> {
    let options = CommandlineOptions::from_args();

    let dir = &options.dir;
    let path = fs::canonicalize(dir).context(dir.to_string())?;
    println!("processing {:?}", path);
    let entries = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| e.is_file());

    let dates: HashMap<PathBuf, NaiveDateTime> = entries
        .filter_map(|e| {
            let from_exif = sopho::exif::date_from_exif(e.clone());
            if let Ok(Some(date)) = from_exif {
                return Some((e, date));
            }
            None
        })
        .collect();

    for date in dates {
        println!("{}: {}", date.0.to_string_lossy(), date.1);
    }
    Ok(())
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "sopho")]
pub struct CommandlineOptions {
    #[structopt(help = "Directory to process", index = 1)]
    pub dir: String,
}
