use crate::data::profile;
use crate::data::training::{TriplePoint, CHARS_PER_LINE, MAX_ERRORS, MIN_CLEAN_PCT};
use crate::data::Theme;
use crate::font;
use crate::style;
use iced::button::{self, Button};
use iced::keyboard::{self, KeyCode};
use iced::{
    Align, Column, Command, Container, Element, Length, Row, Space, Subscription, Text,
    VerticalAlignment,
};
use itertools::{EitherOrBoth, Itertools};

#[derive(Debug)]
pub struct State {
    modifiers: keyboard::Modifiers,
    settings_button: button::State,
    accuracy_metric: TriplePoint,
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyboardEvent(iced::keyboard::Event),
    UserButtonPressed,
    WindowFocused,
    WindowUnocused,
}

pub enum Event {
    Settings,
}

const CHAR_WIDTH: u16 = 10;
const ROW_CHARS: u16 = (CHARS_PER_LINE + MAX_ERRORS - 1) as u16;
const ROW_WIDTH: u16 = CHAR_WIDTH * ROW_CHARS;
const ROW_ERROR_WIDTH: u16 = (MAX_ERRORS - 1) as u16 * CHAR_WIDTH;
const LINE_SPACE: u16 = 10;

impl State {
    pub fn new() -> Self {
        Self {
            modifiers: keyboard::Modifiers::default(),
            settings_button: button::State::new(),
            accuracy_metric: TriplePoint::new(0.5, MIN_CLEAN_PCT, 0.975).unwrap_or_default(),
        }
    }

    pub fn update(
        &mut self,
        profiles: &mut profile::List,
        message: Message,
    ) -> Option<(Command<Message>, Event)> {
        match message {
            Message::KeyboardEvent(keyboard_event) => {
                self.handle_keyboard(profiles, keyboard_event)
            }
            Message::UserButtonPressed => Some((Command::none(), Event::Settings)),
            _ => None,
        }
    }

    pub fn view(&mut self, profiles: &profile::List, theme: &Theme) -> Element<Message> {
        let active_line = Row::with_children(
            profiles
                .session()
                .hits
                .iter()
                .map(|hit| {
                    Text::new(hit.target().to_string())
                        .width(Length::Units(CHAR_WIDTH))
                        .font(font::THIN)
                        .color(if hit.is_dirty() {
                            theme.miss
                        } else {
                            theme.text
                        })
                })
                .chain(
                    profiles
                        .session()
                        .errors
                        .iter()
                        .zip_longest(
                            std::iter::once(&profiles.session().active_hit.target())
                                .chain(profiles.session().targets.iter()),
                        )
                        .map(|result| match result {
                            EitherOrBoth::Left(e) | EitherOrBoth::Both(e, _) => {
                                let c = if *e == ' ' { '\u{2591}' } else { *e };
                                Text::new(c.to_string())
                                    .width(Length::Units(CHAR_WIDTH))
                                    .font(font::MEDIUM)
                                    .color(theme.error)
                            }
                            EitherOrBoth::Right(t) => {
                                Text::new(t.to_string()).width(Length::Units(CHAR_WIDTH))
                            }
                        }),
                )
                .map(|text| text.into())
                .collect(),
        );

        let target_indicator: Element<_> = if profiles.session().errors.is_empty() {
            Row::with_children(vec![
                Space::with_width(Length::Units(
                    profiles.session().hits.len() as u16 * CHAR_WIDTH,
                ))
                .into(),
                Text::new("\u{2015}")
                    .width(Length::Units(CHAR_WIDTH))
                    .height(Length::Units(LINE_SPACE))
                    .vertical_alignment(VerticalAlignment::Center)
                    .color(theme.target)
                    .into(),
            ])
            .into()
        } else {
            Space::with_height(Length::Units(LINE_SPACE)).into()
        };

        let content_active = Column::new()
            .width(Length::Units(ROW_WIDTH))
            .push(active_line)
            .push(target_indicator);

        let content_next = Column::with_children(
            profiles
                .session()
                .next_lines
                .iter()
                .map(|line| {
                    Row::with_children(
                        line.chars()
                            .map(|c| {
                                Text::new(c.to_string())
                                    .width(Length::Units(CHAR_WIDTH))
                                    .into()
                            })
                            .collect(),
                    )
                    .into()
                })
                .collect(),
        )
        .spacing(LINE_SPACE)
        .width(Length::Units(ROW_WIDTH));

        let training = Column::with_children(vec![content_active.into(), content_next.into()])
            .padding([0, 0, 0, ROW_ERROR_WIDTH]);
        let training = Container::new(training)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        let clean_letters = Column::with_children(
            profiles
                .active()
                .state
                .clean_letters()
                .iter()
                .map(|(ch, val)| {
                    Row::new()
                        .push(Text::new(format!("{}", ch)).font(font::LIGHT).size(12))
                        .push(
                            Text::new("\u{25a0}")
                                .color(theme.metric(self.accuracy_metric.value(*val)))
                                .font(font::LIGHT)
                                .size(16),
                        )
                        .align_items(Align::Center)
                        .spacing(5)
                        .into()
                })
                .collect(),
        )
        .spacing(2)
        .padding(5);

        let content = Row::new()
            .push(clean_letters)
            .push(training)
            .width(Length::Fill)
            .height(Length::Fill);

        let settings_button_content = Column::new()
            .push(Text::new(profiles.active().name.clone()).size(14))
            .push(Text::new(profiles.active().layout.to_string()).size(14))
            .width(Length::Fill)
            .align_items(Align::End)
            .spacing(5);

        let settings_button = Button::new(&mut self.settings_button, settings_button_content)
            .on_press(Message::UserButtonPressed)
            .style(style::button::text(theme))
            .padding(10);

        let footer = Row::new()
            .push(Space::with_width(Length::Fill))
            .push(settings_button);

        Column::with_children(vec![content.into(), footer.into()]).into()
    }

    pub fn handle_keyboard(
        &mut self,
        profiles: &mut profile::List,
        event: iced::keyboard::Event,
    ) -> Option<(Command<Message>, Event)> {
        match event {
            keyboard::Event::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                None
            }

            keyboard::Event::KeyPressed { key_code, .. } => match key_code {
                KeyCode::Space => {
                    if let Some(line) = profiles.session_mut().apply_char(' ') {
                        if let Some(words) = profiles.active_mut().add_line(line) {
                            profiles.session_mut().update_words(words)
                        }
                        profiles.session_mut().fill_next_lines();
                    }
                    None
                }
                KeyCode::Escape => None,
                KeyCode::Backspace => {
                    profiles.session_mut().backspace();
                    None
                }
                _ => None,
            },
            keyboard::Event::CharacterReceived(c)
                if c.is_alphanumeric() && !self.modifiers.is_command_pressed() =>
            {
                if let Some(line) = profiles.session_mut().apply_char(c) {
                    if let Some(words) = profiles.active_mut().add_line(line) {
                        profiles.session_mut().update_words(words)
                    }
                    profiles.session_mut().fill_next_lines();
                }
                None
            }
            _ => None,
        }
    }
}

pub fn subscription() -> Subscription<Message> {
    use iced_native::event::{Event, Status};
    use iced_native::window::Event as WindowEvent;

    iced_native::subscription::events_with(|event, status| {
        if status == Status::Captured {
            return None;
        }
        match event {
            Event::Keyboard(keyboard_event) => Some(Message::KeyboardEvent(keyboard_event)),
            Event::Window(WindowEvent::Focused) => Some(Message::WindowFocused),
            Event::Window(WindowEvent::Unfocused) => Some(Message::WindowUnocused),
            _ => None,
        }
    })
}
