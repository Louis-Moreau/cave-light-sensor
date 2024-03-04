use iced::{executor, Alignment, Application, Command, Length, Theme};
use iced::widget::{column, container, pick_list, scrollable, vertical_space, text, row};
use crate::ui::Message::BaudRateSelected;

#[derive(Default)]
pub struct Ui {
    baud_rate : Option<u32>,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateSerialPortList,
    SerialPortSelected(String),
    BaudRateSelected(u32),
    Connect,
    SendCommand(())
}

impl Application for Ui {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self {
            baud_rate: None,
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Sensor data extractor")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            BaudRateSelected(b) => self.baud_rate = Some(b),
            _ => ()
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let pick_list = pick_list(BAUD_RATES.to_vec(), self.baud_rate, BaudRateSelected);

        let logs = scrollable(text("test\ntest\ntest\ntest\ntest\ntest\ntest\ntest\n"));

        let content = row![column![
            vertical_space(30),
            "Which is your favorite language?",
            pick_list,
            vertical_space(30),
        ],logs].width(Length::Fill)
        .align_items(Alignment::Start)
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()

    }
}


const BAUD_RATES: [u32; 4] = [1200,2400,4800,9600];