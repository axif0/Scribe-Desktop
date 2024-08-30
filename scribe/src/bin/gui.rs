use iced::widget::{container, image::Handle, row, text, text_input, Column, Image};
use iced::{
    alignment::Horizontal, executor, window, Alignment, Application, Command, Element, Length,
    Settings, Size, Subscription, Theme,
};
use iced_futures::subscription;
use std::io::Read;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum Message {
    KeyReceived(char),
    NoOp,
}

struct Scribe {
    keys: Arc<Mutex<String>>,
}

impl Default for Scribe {
    fn default() -> Self {
        Scribe {
            keys: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl Application for Scribe {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Scribe")
    }

    fn theme(&self) -> Self::Theme {
        Theme::Light
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::KeyReceived(char) => {
                let mut keys = self.keys.lock().unwrap();
                if char == '\x08' {
                    keys.pop();
                } else {
                    keys.push(char);
                }
            }
            Message::NoOp => todo!(),
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::unfold((), self.keys.clone(), |keys_arc| async move {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next() {
                if let Ok(mut stream) = stream {
                    let mut buffer = [0; 1];
                    if let Ok(_) = stream.read_exact(&mut buffer) {
                        if let Some(received_char) = char::from_u32(buffer[0] as u32) {
                            return (Message::KeyReceived(received_char), keys_arc);
                        } else {
                            println!("Received an invalid character");
                            return (Message::NoOp, keys_arc);
                        }
                    }
                }
            }

            (Message::NoOp, keys_arc)
        })
    }

    fn view(&self) -> Element<Message> {
        let logo_data: &[u8] = include_bytes!("../../ScribeBtnPadBlack.png");
        let logo: Image<Handle> = Image::new(Handle::from_memory(logo_data.to_vec())).width(50);

        let keys = self.keys.lock().unwrap().clone();
        let text_for_translation = text_input("Your translation here ...", &keys);

        let title_row = container(row!(text("Welcome to Scribe").size(30)))
            .width(Length::Fill)
            .align_x(Horizontal::Center);
        let content_row = row!(logo, text_for_translation).align_items(Alignment::Center);

        Column::new()
            .width(Length::Fill)
            .padding(10)
            .spacing(10)
            .push(title_row)
            .push(content_row)
            .into()
    }
}

fn main() -> Result<(), iced::Error> {
    let settings = Settings {
        window: window::Settings {
            size: Size {
                width: 400.0,
                height: 200.0,
            },
            position: window::Position::Centered,
            resizable: true,
            decorations: true,
            ..window::Settings::default()
        },
        ..Settings::default()
    };

    Scribe::run(settings)
}
