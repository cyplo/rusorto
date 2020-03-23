use structopt::StructOpt;
use std::{fs, io};

fn main() -> Result<(), io::Error> {
    let options = CommandlineOptions::from_args();

    let path = fs::canonicalize(options.dir)?;
    println!("processing {:?}", path);
    Ok(())
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "rusorto")]
pub struct CommandlineOptions {
    #[structopt( help = "Directory to process", index = 1 )]
    pub dir: String,
}
