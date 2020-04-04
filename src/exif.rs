
use anyhow::*;
use chrono::NaiveDateTime;
use exif::Tag;
use exif::{Exif, In};
use std::path::PathBuf;

pub fn date_from_exif(entry: PathBuf) -> Result<Option<NaiveDateTime>> {
    let exif_data = load_exif_data(&entry).context(format!("{}", entry.to_string_lossy()))?;
    let date_time_data = exif_data.get_field(Tag::DateTime, In::PRIMARY);
    if let Some(data) = date_time_data {
        let text = format!("{}", data.value.display_as(data.tag));
        let date = parse_exif_date(text).context(format!("{}", entry.to_string_lossy()))?;
        return Ok(Some(date));
    }
    Ok(None)
}

fn load_exif_data(entry: &PathBuf) -> Result<Exif> {
    let file = std::fs::File::open(entry.clone())?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = exifreader.read_from_container(&mut bufreader)?;
    Ok(exif_data)
}

fn parse_exif_date(text: String) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(&text, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| anyhow!("Error parsing date from exif: {}", e))
}
