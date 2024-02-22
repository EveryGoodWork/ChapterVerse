use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

use log::{debug, info, trace, warn, error};
use std::io::stdout;
use std::time::SystemTime;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum PrintCommand {
    Debug,
    Info,
    Trace,
    Warn,
    Error,
}

#[allow(dead_code)]
impl PrintCommand {
    // This method now returns a tuple containing both the title color and the statement color
    fn colors(&self) -> (Color, Color) {
        match self {
            Self::Debug => (Color::Grey, Color::White), // Example: Yellow for title, Cyan for statement
            Self::Info => (Color::Blue, Color::Cyan), // Example: Yellow for title, Cyan for statement
            Self::Trace => (Color::White, Color::Magenta), // Adjust colors as needed
            Self::Warn => (Color::Yellow, Color::White),
            Self::Error => (Color::Red, Color::DarkRed), // Red for title, DarkRed for statement
        }
    }

    fn print_message(&self, level: &str, message: &str) {
        let mut stdout: std::io::Stdout = stdout();
        let (level_color, statement_color) = self.colors();

        // Set title color
        stdout.execute(SetForegroundColor(level_color)).unwrap();
        print!("[{}][{}] ",
            humantime::format_rfc3339_seconds(SystemTime::now()),
            level,
            );

        // Set statement color
        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", message);

        stdout.execute(ResetColor).unwrap();
    }
}

pub fn debug(message: &str) {
    debug!("{}", message);
    PrintCommand::Debug.print_message("Debug", message);
}

pub fn info(message: &str) {
    info!("{}", message);
    PrintCommand::Info.print_message("Info", message);
}

pub fn trace(message: &str) {
    trace!("{}", message);
    PrintCommand::Trace.print_message("Trace", message);
}

pub fn warn(message: &str) {
    warn!("{}", message);
    PrintCommand::Warn.print_message("Warn", message);
}

pub fn error(message: &str) {
    error!("{}", message);
    PrintCommand::Error.print_message("Error", message);
}

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
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("logs/log.log")?)
        .apply()?;
    Ok(())
}

#[cfg(test)]
mod unittests {
    use super::*;

    #[test]
    fn print_current_message() {
        debug("Testing Debug Message");
        info("Testing Info Message");
        trace("Testing Trace Message");
        warn("Testing Warning Message");
        error("Testing Error Message");
    }
}
