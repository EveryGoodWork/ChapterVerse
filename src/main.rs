use helpers::import_bibles::start_bible_import;

use crate::helpers::env_variables::get_env_variable;
use std::{env, fs};
use log::{debug, error, info, trace, warn};

use std::time::SystemTime;
use std::io;
use humantime;

mod helpers;
mod scripture;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
    .format(|out, message, record| {
        out.finish(format_args!(
            "[{} {} {}] {}",
            humantime::format_rfc3339_seconds(SystemTime::now()),
            record.level(),
            record.target(),
            message,
        ))
    })
    .chain(
        fern::Dispatch::new()
            // by default only accept warn messages
            .level(log::LevelFilter::Warn)
            // accept info messages from the current crate too
            .level_for("chapterverse", log::LevelFilter::Trace)
            // `io::Stdout`, `io::Stderr` and `io::File` can be directly passed in.
            .chain(io::stdout()),
    )
    .chain(
        fern::Dispatch::new()
            // output all messages
            .level(log::LevelFilter::Trace)
            .chain(fern::log_file("logs/log.log")?)
    )
    .chain(
        fern::Dispatch::new()
            .level(log::LevelFilter::Error)
            .chain(io::stderr()),
    )
    .apply()?;

    Ok(())
}

fn main() {
    setup_logger().unwrap();
    
    info!("Jesus is Lord!");
    info!("Version {}", env!("CARGO_PKG_VERSION"));
    info!("What is the Gospel? Gospel means good news! The bad news is we have all sinned and
        deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then
        raised on the third day, according to the scriptures. He ascended into heaven and right now
        is seated at the Father's right hand. Jesus said, \"I am the way, and the truth, and the
        life. No one comes to the Father except through me. The time is fulfilled, and the kingdom
        of God is at hand; repent and believe in the gospel.\"");

    let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");

    let bibles_directory = env::current_dir()
        .expect("Failed to get current directory")
        .join(import_bibles_path);

    debug!("Bibles Directory {}", &bibles_directory.to_string_lossy());
    
    let files = fs::read_dir(&bibles_directory).expect("Failed to read directory");

    for file in files {
        let entry = file.expect("Failed to read entry");
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("csv") {
            match start_bible_import(&entry.path().to_string_lossy()) {
                Ok(imported_bible) => {
                    info!("Bible Imported {}", &entry.path().display().to_string());
                    debug!("Number of verses imported {}", &imported_bible.len().to_string());
                    debug!("2 Timothy 3:16: {}", &imported_bible.get_scripture("55:3:16").0.then(||
                            imported_bible.get_scripture("55:3:16").1).unwrap_or_else(|| "Verse not
                            found".to_string()));



                },
                Err(err) => {
                    error!("Error running import: {}", err);
                }
            }
        }
    }
    
}
