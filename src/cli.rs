use std::io::{stdout, Write};
use structopt::StructOpt;
use termion;
pub use termion::color as Color;

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
    print!("{}", termion::cursor::Save);
}

pub fn print_status_line<S: AsRef<str>>(color: &dyn termion::color::Color, text: S) {
    let text = text.as_ref();

    stdout().write_all(
        format!(
            "{}{}{}⬤{} {}",
            termion::cursor::Restore,
            termion::clear::CurrentLine,
            termion::color::Fg(color),
            termion::color::Fg(termion::color::Reset),
            text
        )
        .as_bytes(),
    );
    stdout().flush().unwrap();
}
