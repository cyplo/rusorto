use std::{fs, io};
use std::collections::HashMap;
use std::iter::Map;
use std::path::PathBuf;

use anyhow::*;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use exif::In;
use exif::Tag;
use rayon::prelude::*;
use structopt::StructOpt;
use walkdir::WalkDir;

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
            return Some((e, date))
        }
        None
    }  ).collect();

    for date in dates {
        println!("{}: {}", date.0.to_string_lossy(), date.1);
    }
    Ok(())
}

fn date_from_exif(entry: PathBuf) -> Result<Option<NaiveDateTime>> {
    let file = std::fs::File::open(entry.clone())?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = exifreader.read_from_container(&mut bufreader)?;
    let date_time_data = exif_data.get_field(Tag::DateTime, In::PRIMARY);
    if let Some(data) = date_time_data {
        let text = format!("{}", data.value.display_as(data.tag));
        let date = parse_date(text).context(format!("{}", entry.to_string_lossy()))?;
        return Ok(Some(date))
    }
    Ok(None)
}

fn parse_date(text: String) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(&text, "%Y-%m-%d %H:%M:%S").map_err(|e| anyhow!("Error parsing date from exif: {}", e))
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "sopho")]
pub struct CommandlineOptions {
    #[structopt(help = "Directory to process", index = 1)]
    pub dir: String,
}
