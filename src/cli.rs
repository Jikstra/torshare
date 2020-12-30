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

    Download {
        #[structopt(flatten)]
        general_options: GeneralOptions,
        url: String,
    },

    Share {
        #[structopt(flatten)]
        general_options: GeneralOptions,
        file_or_folder: String,
    },
}

#[derive(Debug, StructOpt)]
pub struct GeneralOptions {
    /// Activate debug mode
    #[structopt(short, long, env = "DEBUG")]
    pub debug: bool,
    #[structopt(long, env = "TOR_DIR")]
    pub tor_dir: Option<String>,
    #[structopt(long, env = "TOR_DIR_HS")]
    pub tor_dir_hs: Option<String>,
}

pub fn save_cursor_position() {
    //print!("{}", termion::cursor::Save);
}

pub fn print_status_line<S: AsRef<str>>(color: &dyn termion::color::Color, text: S) {
    let text = text.as_ref();

    stdout().write_all(
        format!(
            //"{}{}{}{}⬤{} {}\n",
            "{}⬤{} {}\n",
            //termion::cursor::Up(1), termion::cursor::Left(0), termion::clear::AfterCursor,
            termion::color::Fg(color),
            termion::color::Fg(termion::color::Reset),
            text
        )
        .as_bytes(),
    );
    stdout().flush().unwrap();
}
