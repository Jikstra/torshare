use std::io::{stdout, stdin, Write};
use structopt::StructOpt;
use termion::{self, event::Key};
use termion::input::TermRead;
pub use termion::color as Color;

use crate::{download_file::DownloadOptions, share::ShareOptions};



#[derive(Debug, StructOpt)]
#[structopt(
    name = "TorShare",
    about = "A CLI tool to share and download files and folders through tor."
)]
pub enum CliOptions {
    Download {
        #[structopt(flatten)]
        download_options: DownloadOptions,
    },
    Share {
        #[structopt(flatten)]
        share_options: ShareOptions
    }
    
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

pub fn prompt<S: AsRef<str>>(color: &dyn termion::color::Color, text: S) -> bool {
    let stdin = stdin();

    let text = text.as_ref();

    stdout().write_all(
        format!(
            //"{}{}{}{}⬤{} {}\n",
            "{}⬤{} {}",
            //termion::cursor::Up(1), termion::cursor::Left(0), termion::clear::AfterCursor,
            termion::color::Fg(color),
            termion::color::Fg(termion::color::Reset),
            text
        )
        .as_bytes(),
    );
    stdout().flush().unwrap();
    

    let mut yes = false;
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('y') => {
                yes = true;
                break;
            },
            Key::Char('n') => break,
            _ => {} 
        }
    };

    yes
}
