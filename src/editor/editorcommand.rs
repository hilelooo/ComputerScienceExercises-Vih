use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::convert::TryFrom;
use super::terminal::Size;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

#[derive(Clone, Copy)]
pub enum EditorCommand {
    Key(char),
    Resize(Size),
    Escape,
    Other,
    Delete,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Tab,
    Enter,
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
                (KeyCode::Delete, _) => Ok(Self::Delete),
                (KeyCode::Backspace, _) => Ok(Self::Backspace),
                (KeyCode::Up, _) => Ok(Self::Up),
                (KeyCode::Down, _) => Ok(Self::Down),
                (KeyCode::Left, _) => Ok(Self::Left),
                (KeyCode::Right, _) => Ok(Self::Right),
                (KeyCode::Tab, _) => Ok(Self::Tab),
                (KeyCode::Enter, _) => Ok(Self::Enter),
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
