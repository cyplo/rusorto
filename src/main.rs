use std::{fs, io};
use structopt::StructOpt;
use walkdir::WalkDir;

fn main() -> Result<(), io::Error> {
    let options = CommandlineOptions::from_args();

    let path = fs::canonicalize(options.dir)?;
    println!("processing {:?}", path);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        // get date from exif
        // get date from file name
        // if they don't agree - use exif
        // if there's no exif - use file name
    }
    Ok(())
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "sopho")]
pub struct CommandlineOptions {
    #[structopt(help = "Directory to process", index = 1)]
    pub dir: String,
}
