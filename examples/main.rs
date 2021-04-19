#[macro_use]
extern crate failure;

use failure::Error;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::Terminal;
use tui_image::{ColorMode, Image};

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Please give an image file as an argument.")]
struct UsageError;

fn run() -> Result<(), Error> {
    let img_path = std::env::args().nth(1).ok_or_else(|| UsageError {})?;

    let img = image::open(img_path)?.to_rgba();

    let stdout = std::io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal.draw(|f| {
        let size = f.size();

        f.render_widget(
            Image::with_img(img).color_mode(ColorMode::Rgb),
            Rect {
                x: 0,
                y: 0,
                width: size.width,
                height: size.height,
            },
        )
    })?)
}
