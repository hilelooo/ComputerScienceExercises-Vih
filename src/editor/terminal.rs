use std::io::{stdout, Error, Write};
use crossterm::cursor;
use crossterm::queue;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal;
use crossterm::style;

pub struct Terminal;

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize
}

#[derive(Copy, Clone, Default)]
pub struct Coords {
    pub row: usize,
    pub col: usize,
}

impl Coords {
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self {
            row: self.row.saturating_sub(other.row),
            col: self.col.saturating_sub(other.col),
        }
    }
}

impl Terminal {

    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(())
    }

    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn print_row(row: usize, line_text: &str, selected_text: Option<(usize,usize)>) -> Result<(), Error> {
        Self::move_caret_to(Coords {row, col:0})?;
        Self::clear_line()?;
        Self::print(line_text, selected_text)?;
        Ok(())
    }

    pub fn enter_alternate() -> Result<(), Error>{
        queue!(stdout(), EnterAlternateScreen)?;
        Ok(())
    }
    
    pub fn leave_alternate() -> Result<(), Error>{
        queue!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn move_caret_to(coords: Coords) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        queue!(stdout(), cursor::MoveTo(coords.col as u16, coords.row as u16))?;
        Ok(())
    }

    pub fn size() -> Result<Size, Error> {
        let (width_u16, height_u16) = terminal::size()?;
        #[allow(clippy::as_conversions)]
        let height = height_u16 as usize;
        #[allow(clippy::as_conversions)]
        let width = width_u16 as usize;
        Ok(Size { width, height} )
    }

    pub fn hide_caret() -> Result<(), Error> {
        queue!(stdout(), cursor::Hide)?;
        Ok(())
    }

    pub fn show_caret() -> Result<(), Error> {
        queue!(stdout(), cursor::Show)?;
        Ok(())
    }

    pub fn print(s : &str, selected_text: Option<(usize, usize)>) -> Result<(), Error> {
        match selected_text {
            None => {
                queue!(stdout(), style::Print(s))?;
            }
            Some((sel_start, sel_end)) => {
                let (left, rest) = s.split_at(sel_start);
                // FIXME
                let (mid, right) = rest.split_at(sel_end - sel_start);

                queue!(stdout(), style::Print(left))?;
                queue!(
                    stdout(),
                    style::SetBackgroundColor(style::Color::DarkBlue),
                    style::Print(mid),
                    style::ResetColor
                )?;
                queue!(stdout(), style::Print(right))?;
            }
        }
        Ok(())
    }
    
    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
}
