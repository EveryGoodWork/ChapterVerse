use log::{debug, info, trace, warn, error};
use std::time::SystemTime;

pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .chain(fern::log_file("logs/log.log")?)
        .apply()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, Read};
    use std::path::Path;
    use log::info;

    #[test]
    fn test_logging() -> io::Result<()> {
        // Remove the log file if it exists
        // This might delete logs one intends to keep
        let log_path = Path::new("logs/log.log");
        std::fs::remove_file(log_path)?;

        setup_logger().expect("Failed to initialize logger");

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
