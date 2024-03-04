use iced::{Application, Settings};
use ui::Ui;

mod ui;

fn main() -> iced::Result{
    Ui::run(Settings::default())
}
