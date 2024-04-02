use iced::{executor, Alignment, Application, Command, Length, Theme};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, vertical_space};

#[derive(Default)]
pub struct Ui {
    
}

#[derive(Debug, Clone)]
pub enum Message {
   
}

impl Application for Ui {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self {
            
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
           
            _ => ()
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
       row![
        column![
            row![text("PICK LIST HERE"),button("Connect")],
            button("Connect")

        ].padding(20)
        .align_items(Alignment::Center)


       ].padding(20)
       .align_items(Alignment::Center)
       .into()

    }
}
