use iced::executor;
use iced::keyboard;
use iced::{
    self, Align, Application, Clipboard, Column, Command, Container, Element, Length, Row,
    Settings, Subscription, Text,
};
use iced_native;

mod data;

pub fn main() -> iced::Result {
    let freq = data::freq::Freq::load();

    Linkage::run(Settings {
        flags: Flags {
            freq: freq.unwrap_or_default(),
        },
        ..Settings::default()
    })
}

#[derive(Debug, Default)]
struct Linkage {
    should_exit: bool,
    freq: data::freq::Freq,
    words: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Event(iced_native::Event),
    Exit,
}

#[derive(Debug, Default)]
struct Flags {
    freq: data::freq::Freq,
}

impl Application for Linkage {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Flags) -> (Linkage, Command<Message>) {
        let linkage = Linkage {
            should_exit: false,
            freq: flags.freq,
            words: Vec::new(),
        };
        (linkage, Command::none())
    }

    fn title(&self) -> String {
        String::from("Linkage")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Event(event) => {
                self.handle_event(event);
            }
            Message::Exit => {
                self.should_exit = true;
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn view(&mut self) -> Element<Message> {
        const SPACE: u16 = 10;
        let row = Row::with_children(
            self.words
                .iter()
                .map(|word| Text::new(word.clone()).into())
                .collect(),
        )
        .spacing(SPACE);

        let instruction = Text::new("Press <Enter> for random words.").size(13);

        let content = Column::with_children(vec![row.into(), instruction.into()])
            .spacing(10)
            .align_items(Align::Center);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

impl Linkage {
    fn handle_event(&mut self, event: iced_native::Event) {
        match event {
            iced_native::Event::Keyboard(keyboard_event) => self.handle_keyboard(keyboard_event),
            iced_native::Event::Mouse(_mouse_event) => {}
            _ => {}
        }
    }

    fn handle_keyboard(&mut self, event: keyboard::Event) {
        use iced::keyboard::KeyCode;

        let Linkage { words, freq, .. } = self;

        match event {
            keyboard::Event::KeyPressed { key_code, .. } => match key_code {
                KeyCode::Enter => {
                    *words = (0..10).map(|_| freq.random_word()).collect::<Vec<String>>();
                }
                _ => {}
            },
            _ => {}
        }
    }
}
