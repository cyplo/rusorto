use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

pub fn get(path: PathBuf) -> Option<NaiveDateTime> {
    lazy_static! {
        static ref alltogether: Regex = Regex::new(
            r"(?x)
(?P<year>\d{4})
(?P<month>\d{2})
(?P<day>\d{2})
"
        )
        .unwrap();
    }
    let name = path.to_string_lossy();
    let capture_groups = alltogether.captures(&name);

    None
}
