pub use crossterm::style::Color;
use crossterm::{
    cursor, execute,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::stdout;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "TorShare",
    about = "A CLI tool to share and download files and folders through tor."
)]
pub enum CliOptions {
    #[structopt(flatten)]
    GeneralOptions(GeneralOptions),

    Download {
        url: String,
    },

    Share {
        file_or_folder: String,
    },
}

#[derive(Debug, StructOpt)]
pub struct GeneralOptions {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
}

pub fn save_cursor_position() {
    execute!(stdout(), cursor::SavePosition);
}

pub fn print_status_line<S: AsRef<str>>(color: Color, text: S) {
    let text = text.as_ref();
    execute!(
        stdout(),
        cursor::RestorePosition,
        terminal::Clear(terminal::ClearType::All),
        SetForegroundColor(color),
        Print("⬤"),
        ResetColor,
        Print(" "),
        Print(text)
    );
}
