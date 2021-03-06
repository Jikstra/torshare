use std::io::{stdout, Write};
use structopt::StructOpt;
use termion;
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
