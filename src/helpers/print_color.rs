use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

use std::io::stdout;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum PrintCommand {
    System,
    Info,
    Details,
    UnitTest,
    Issue,
    Error,
}
#[allow(dead_code)]
impl PrintCommand {
    // This method now returns a tuple containing both the title color and the statement color
    fn colors(&self) -> (Color, Color) {
        match self {
            Self::System => (Color::Green, Color::Cyan), // Example: Yellow for title, Cyan for statement
            Self::Info => (Color::Grey, Color::White), // Example: Yellow for title, Cyan for statement
            Self::Details => (Color::Grey, Color::White), // Example: Yellow for title, Cyan for statement
            Self::UnitTest => (Color::White, Color::Magenta), // Adjust colors as needed
            Self::Issue => (Color::Yellow, Color::White),
            Self::Error => (Color::Red, Color::DarkRed), // Red for title, DarkRed for statement
        }
    }

    pub fn print_message(&self, title: &str, message: &str) {
        let mut stdout: std::io::Stdout = stdout();
        let (title_color, statement_color) = self.colors();

        // Set title color
        stdout.execute(SetForegroundColor(title_color)).unwrap();
        print!("{}: ", title);

        // Set statement color
        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", message);

        stdout.execute(ResetColor).unwrap();
    }
}

#[cfg(test)]
mod unittests {
    use super::*;

    #[test]
    fn print_current_message() {
        PrintCommand::System.print_message("Unit Test", "Testing System Message");
        PrintCommand::Issue.print_message("Unit Test", "Testing Issue Message");
        PrintCommand::UnitTest.print_message("Unit Test", "Testing UnitTest Message");
    }
}
