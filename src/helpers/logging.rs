use std::env;
use log::{debug, info, trace, warn, error};
use std::time::SystemTime;

pub fn setup_logger() -> Result<String, fern::InitError> {
    let verbose_flag = env::args().any(|arg| arg == "-v");
    let very_verbose_flag = env::args().any(|arg| arg == "-vv");
    let level = if verbose_flag {
        log::LevelFilter::Info
    } else if very_verbose_flag {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };
    let log_path = format!(
        "logs/log-{}.log", humantime::format_rfc3339_seconds(
            SystemTime::now()
            ),
    );
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(
            fern::log_file(&log_path)?
            )
        .apply()?;
    Ok(log_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, Read};
    use log::info;

    #[test]
    fn test_logging() -> io::Result<()> {
        let log_path = setup_logger().expect("Failed to initialize logger");

        debug!("debug");
        info!("info");
        trace!("trace");
        warn!("warn");
        error!("error");

        let mut file = File::open(&log_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Checks that the log file contains the expected messages
        assert!(!contents.contains("debug"));
        assert!(!contents.contains("info"));
        assert!(!contents.contains("trace"));
        assert!(contents.contains("warn"));
        assert!(contents.contains("error"));

        // Removes the log file
        std::fs::remove_file(log_path)?;

        Ok(())
    }
}
