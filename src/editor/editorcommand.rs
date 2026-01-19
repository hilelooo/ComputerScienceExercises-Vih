use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::convert::TryFrom;
use super::terminal::Size;

pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

pub enum EditorCommand {
    Key(char),
    Resize(Size),
    Escape,
    Other,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char(_), KeyModifiers::NONE | KeyModifiers::SHIFT) => 
                    if let KeyCode::Char(c) = code {Ok(Self::Key(c))} else {Ok(Self::Other)},
                (KeyCode::Esc, _) => Ok(Self::Escape),
                _ => Ok(Self::Other),
            },
            Event::Resize(width_u16, height_u16) => {
                #[allow(clippy::as_conversions)]
                let height = height_u16 as usize;
                #[allow(clippy::as_conversions)]
                let width = width_u16 as usize;
                Ok(Self::Resize(Size {width, height}))
            },
            _ => Err(format!("unsupported event: {event:?}")),
        }
    }
}
