use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::convert::TryFrom;
use super::terminal::Size;
use super::view::Bmode;

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
    Other,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Left | KeyCode::Char('h'), KeyModifiers::NONE) =>
                    Ok(Self::Move(Direction::Left)),
                (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) =>
                    Ok(Self::Move(Direction::Up)),
                (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) =>
                    Ok(Self::Move(Direction::Down)),
                (KeyCode::Right | KeyCode::Char('l'), KeyModifiers::NONE) =>
                    Ok(Self::Move(Direction::Right)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                _ => Ok(Other),
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
