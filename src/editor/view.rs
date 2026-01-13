use crossterm::event::Event;
use std::cmp::min;
use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Size, Terminal, Coords}
};
use self::line::Line;

mod buffer;
use buffer::Buffer;
mod line;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Bmode {
    Normal,
    Insert,
    Visual,
    Replace,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    text_location: Location,
    scroll_offset: Coords,
    size: Size,
    bmode: Bmode,
}

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

impl View {
    pub fn render(&mut self) {
        if !self.needs_redraw {return;}
        let Size {height, width} = self.size;
        if height == 0 || width == 0 {return;}
        if self.buffer.is_empty() {
            self.render_welcome_screen();
        } else {
            self.render_buffer();
        }
        self.needs_redraw = false;
    }

    pub fn handle_command(&mut self, event: Event) -> bool {
        //faire en sorte que bmode soit pris en compte (ne peut pas etre argument de tryfrom)
        //peut etre retirer le editorcommandmove et remplacer par lettre normale 
        //puis mettre move sur view plutot que sur editorcommand
        match self.bmode {
            Normal => {
                match EditorCommand::try_from(event) {
                    EditorCommand::Resize(size) => self.resize(size),
                    EditorCommand::Key(c) => {
                        match c {
                            'h' => self.move_text_location(Direction::Left),
                            'j' => self.move_text_location(Direction::Down),
                            'k' => self.move_text_location(Direction::Up),
                            'l' => self.move_text_location(Direction::Right),
                        }
                    }
                    _ => {},
                }
            },
            Insert => {}
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
        self.needs_redraw = true;
    }

    fn render_line(row: usize, line_text: &str) {
        let result = Terminal::print_row(row, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    fn render_buffer(&self) {
        let Size {height, width} = self.size;
        for row in 0..height {
            if let Some(e) = self.buffer.lines.get(row.saturating_add(self.scroll_offset.row)) {
                let xbound1 = self.scroll_offset.col;
                let xbound2 = self.scroll_offset.col + width;
                Self::render_line(row, &e.get_visible_graphemes(xbound1..xbound2));
            } else  {
                Self::render_line(row, "~");
            }
        }
    }

    fn render_welcome_screen(&self) {
        let Size {height, ..} = self.size;
        for row in 0..height {
            let _ = Terminal::clear_line();
            if row == height / 2 {
                self.draw_welcome_message();
            } else  {
                Self::draw_empty_row();
            }
            if row.saturating_add(1) < height {
                let _ = Terminal::move_caret_to(Coords {row: row+1, col: 0} );
            }
        }
    }

    fn draw_empty_row() {
        let _ = Terminal::print("~");
    }

    fn draw_welcome_message(&self) {
        let mut welcome_msg = format!("{NAME} version {VERSION}");
        let width: usize = self.size.width;
        let len = welcome_msg.len(); 
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len)) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{spaces}{welcome_msg}");
        welcome_msg.truncate(width);
        let _ = Terminal::print(&welcome_msg);
    }

    fn resize(&mut self, size: Size){
        self.size = size;
        self.scroll_text_location_into_view();
        self.needs_redraw = true;                
    }

    fn scroll_vertically(&mut self, to: usize) {
        let Size { height, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };
        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_horizontally(&mut self, to: usize) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };
        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_text_location_into_view(&mut self) {
        let Coords { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    pub fn caret_position(&self) -> Coords {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    /* unused (i think)
    pub fn get_position(&self) -> Coords {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }*/

    pub fn text_location_to_position(&self) -> Coords {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });
        Coords {col, row}
    }

     fn move_text_location(&mut self, direction: &Direction) {
         let Size { height, .. } = self.size;
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }
    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }
    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index += 1;
        }
    }
    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index -= 1;
        }
    }
    
    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }
    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                min(line.grapheme_count(), self.text_location.grapheme_index)
            });
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(self.text_location.line_index, self.buffer.height());
    }

}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Coords::default(),
            bmode: Bmode::Normal,
        }
    }
}
