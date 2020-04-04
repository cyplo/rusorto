use anyhow::*;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use rexif::{ExifTag, TagValue};
use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;
use walkdir::WalkDir;
use std::iter::Map;
use std::collections::HashMap;

fn main() -> Result<()> {
    let options = CommandlineOptions::from_args();

    let path = fs::canonicalize(options.dir)?;
    println!("processing {:?}", path);
    let entries = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| e.is_file());

    let dates : HashMap <PathBuf, NaiveDateTime> = entries.filter_map( |e| {
        let from_exif = date_from_exif(e.clone());
        if let Ok(Some(date)) = from_exif {
            return Some((e, date));
        }
        return None;
    }  ).collect();

    for date in dates {
        println!("{}: {}", date.0.to_string_lossy(), date.1);
    }
    Ok(())
}

fn date_from_exif(entry: PathBuf) -> Result<Option<NaiveDateTime>> {
    let exif =
        rexif::parse_file(entry.as_path()).context(format!("path: {}", entry.to_string_lossy()))?;
    let date = exif.entries.into_iter().find_map(|e| match e.tag {
        ExifTag::DateTime => Some(e),
        _ => None,
    });

    match date {
        None => Ok(None),
        Some(date) => Ok(Some(parse_date(date.value).context(format!("{}", entry.to_string_lossy()))?)),
    }
}

fn parse_date(value: TagValue) -> Result<NaiveDateTime> {
    match value {
        TagValue::Ascii(text) => {
            NaiveDateTime::parse_from_str(&text, "%Y:%m:%d %H:%M:%S").map_err(|e| anyhow!("Error parsing date from exif: {}", e))
        }
        _ => Err(anyhow!("Tag value is not text")),
    }
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "sopho")]
pub struct CommandlineOptions {
    #[structopt(help = "Directory to process", index = 1)]
    pub dir: String,
}
