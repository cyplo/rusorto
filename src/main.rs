use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use structopt::StructOpt;
use futures::future::LocalBoxFuture;

use async_std::{
    fs::{self, *},
    path::*,
    prelude::*,
};
use walkdir::WalkDir;
use futures::{Future, FutureExt};

mod from;

fn main() -> Result<()> {
    let options = CommandlineOptions::from_args();

    let dir = &options.dir;
    let path = std::fs::canonicalize(dir).context(dir.to_string())?;
    println!("processing {:?}", path);
    let entries = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| e.is_file());

    let dates: HashMap<std::path::PathBuf, NaiveDateTime> = entries
        .filter_map(|e| {
            if let Ok(Some(date)) = crate::from::exif::get(e.clone()) {
                return Some((e, date));
            }
            if let Some(date) = crate::from::filename::get(e.clone()) {
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

async fn walkdir<R>(path: impl AsRef<Path>, mut cb: impl FnMut(DirEntry) -> R)
where
    R: Future<Output = ()>,
{
    fn walkdir_inner<'a, R>(
        path: &'a Path,
        cb: &'a mut dyn FnMut(DirEntry) -> R,
    ) -> LocalBoxFuture<'a, ()>
    where
        R: Future<Output = ()>,
    {
        async move {
            let mut entries = fs::read_dir(path).await.unwrap();

            while let Some(path) = entries.next().await {
                let entry = path.unwrap();
                let path = entry.path();
                if path.is_file().await {
                    cb(entry).await
                } else {
                    walkdir_inner(&path, cb).await
                }
            }
        }
        .boxed_local()
    }

    walkdir_inner(path.as_ref(), &mut cb).await
}

#[derive(Clone, StructOpt, Debug)]
#[structopt(name = "sopho")]
pub struct CommandlineOptions {
    #[structopt(help = "Directory to process", index = 1)]
    pub dir: String,
}
