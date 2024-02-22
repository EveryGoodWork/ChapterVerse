use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

use std::io::stdout;

#[derive(PartialEq, Debug)]
pub enum PrintCommand {
    System,
    Info,
    Issue,
    Error,
}

impl PrintCommand {
    fn colors(&self) -> (Color, Color) {
        match self {
            Self::System => (Color::Green, Color::Cyan),
            Self::Info => (Color::Green, Color::Magenta),
            Self::Issue => (Color::Yellow, Color::Magenta),
            Self::Error => (Color::Red, Color::DarkRed),
        }
    }

    pub fn print_message(&self, title: &str, message: &str) {
        let mut stdout: std::io::Stdout = stdout();
        let (title_color, statement_color) = self.colors();
        stdout.execute(SetForegroundColor(title_color)).ok();
        print!("{}: ", title);
        stdout.execute(SetForegroundColor(statement_color)).ok();
        println!("{}", message);
        stdout.execute(ResetColor).ok();
    }
}

#[cfg(test)]
mod unittests {
    use super::*;

    #[test]
    fn print_current_message() {
        PrintCommand::System.print_message("Unit Test", "Testing System Message");
        PrintCommand::Issue.print_message("Unit Test", "Testing Issue Message");
        PrintCommand::Info.print_message("Unit Test", "Testing Info Message");
        PrintCommand::Error.print_message("Unit Test", "Testing Error Message");
    }
}
