use crossterm::event::Event;
use std::cmp::min;
use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Size, Terminal, Coords},
    DocumentStatus,
};
use self::line::Line;

mod buffer;
mod selection;
use selection::Selection;
use buffer::Buffer;
mod line;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum Bmode {
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
    selection: Selection,
    bmode: Bmode,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
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
        match self.bmode {
            Bmode::Normal => {
                match EditorCommand::try_from(event) {
                    Ok(EditorCommand::Resize(size)) =>self.resize(size),
                    Ok(EditorCommand::Up) => self.move_text_location(Direction::Up),
                    Ok(EditorCommand::Down) => self.move_text_location(Direction::Down),
                    Ok(EditorCommand::Left) => self.move_text_location(Direction::Left),
                    Ok(EditorCommand::Right) => self.move_text_location(Direction::Right),
                    Ok(EditorCommand::Key(c)) => {
                        match c {
                            'h' => self.move_text_location(Direction::Left),
                            'j' => self.move_text_location(Direction::Down),
                            'k' => self.move_text_location(Direction::Up),
                            'l' => self.move_text_location(Direction::Right),
                            'x' => self.delete(),
                            'X' => self.backspace(),
                            's' => self.save(),
                            'i' => self.bmode = Bmode::Insert,
                            'r' => self.bmode = Bmode::Replace,
                            'v' => self.start_selection(),
                            'q' => return true,
                            'z' => self.center_cursor(),
                            _ => {},
                        }
                    }
                    _ => {},
                }
            },
            Bmode::Insert => {
                match EditorCommand::try_from(event) {
                    Ok(EditorCommand::Escape) => self.bmode = Bmode::Normal,
                    Ok(EditorCommand::Key(c)) => self.insert_char(c),
                    Ok(EditorCommand::Delete) => self.delete(),
                    Ok(EditorCommand::Backspace) => self.backspace(),
                    Ok(EditorCommand::Up) => self.move_text_location(Direction::Up),
                    Ok(EditorCommand::Down) => self.move_text_location(Direction::Down),
                    Ok(EditorCommand::Left) => self.move_text_location(Direction::Left),
                    Ok(EditorCommand::Right) => self.move_text_location(Direction::Right),
                    Ok(EditorCommand::Tab) => {self.insert_char(' ');self.insert_char(' ');},
                    Ok(EditorCommand::Enter) => self.insert_line(),
                    _ => {},
                }
            },
            Bmode::Replace => {
                match EditorCommand::try_from(event) {
                    Ok(EditorCommand::Escape) => self.bmode = Bmode::Normal,
                    Ok(EditorCommand::Key(c)) => {self.delete(); self.insert_char(c);},
                    _ => {},
                }
            },
            Bmode::Visual => {self.needs_redraw = true;
                match EditorCommand::try_from(event) {
                    Ok(EditorCommand::Escape) => self.exit_selection(),
                    Ok(EditorCommand::Up) => self.move_text_location(Direction::Up),
                    Ok(EditorCommand::Down) => self.move_text_location(Direction::Down),
                    Ok(EditorCommand::Left) => self.move_text_location(Direction::Left),
                    Ok(EditorCommand::Right) => self.move_text_location(Direction::Right),
                    Ok(EditorCommand::Key(c)) => {
                        match c {
                            'h' => self.move_text_location(Direction::Left),
                            'j' => self.move_text_location(Direction::Down),
                            'k' => self.move_text_location(Direction::Up),
                            'l' => self.move_text_location(Direction::Right),
                            _ => {},
                        }
                    },
                    _ => {},
                }
            },
        }
        false
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.lines.len(),
            current_line_index: self.text_location.line_index,
            filename: self.buffer.filename.clone(),
            is_modified: self.buffer.dirty,
            bmode_string: self.bmode.as_str(),
        }
    }

    fn start_selection(&mut self) {
        self.selection.active = true;
        self.bmode = Bmode::Visual;
        self.selection.anchor = self.text_location;
    }

    fn exit_selection(&mut self) {
        self.selection.active = false;
        self.bmode = Bmode::Normal;
    }

    fn process_selection(&self) -> Option<(Location, Location)> {
        if !self.selection.active {
            return None;
        }

        let (a, b) = (self.selection.anchor, self.text_location);

        if (a.line_index, a.grapheme_index) <= (b.line_index, b.grapheme_index) {
            Some((a, b))
        } else {
            Some((b, a))
        }
    }

    fn is_selected(&self, line_index: usize, grapheme_index: usize) -> bool {
        if let Some((start, end)) = self.process_selection() {
            (line_index, grapheme_index) >= (start.line_index, start.grapheme_index)
                && (line_index, grapheme_index) < (end.line_index, end.grapheme_index)
        } else {
            false
        }
    }

    fn center_cursor(&mut self) {
        let Size { height, .. } = self.size;
        if self.text_location.line_index.saturating_sub(self.scroll_offset.row) < height/2 {
            self.scroll_vertically(self.text_location.line_index.saturating_sub(height/2));
        } else {
            self.scroll_vertically(self.text_location.line_index.saturating_add(height/2));
        }
    }

    fn insert_char(&mut self, c: char) {
        let old_len = self.buffer.lines.get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        self.buffer.insert_char(c, self.text_location);
        let len = self.buffer.lines.get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        if len-old_len > 0 {
            self.move_text_location(Direction::Right);
        }
        self.needs_redraw = true;
    }

    fn insert_line(&mut self) {
        self.buffer.insert_line(self.text_location);
        self.move_text_location(Direction::Down);
        self.move_to_start_of_line();
        self.needs_redraw = true;
    }

    fn delete(&mut self) {
        self.buffer.delete(self.text_location);
        self.needs_redraw = true;
    }

    fn backspace(&mut self) {
        if self.text_location.line_index != 0 || self.text_location.grapheme_index != 0{
            if self.text_location.grapheme_index == 0 {
                self.move_up(1);
                self.move_to_end_of_line();
            } else {
                self.move_left();
            }
            self.delete();
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
        self.needs_redraw = true;
    }

    pub fn save(&mut self) {
        let _ = self.buffer.save();
    }

    fn render_line(row: usize, line_text: &str, selected_text: Option<(usize, usize)>) {
        let result = Terminal::print_row(row, line_text, selected_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    fn render_buffer(&self) {
        let Size {height, width} = self.size;
        for row in 0..height {
            if let Some(e) = self.buffer.lines.get(row.saturating_add(self.scroll_offset.row)) {
                let xbound1 = self.scroll_offset.col;
                let xbound2 = self.scroll_offset.col + width;
                let mut firstselec = None;
                let mut lastselec = None;
                for col in xbound1..xbound2 {
                    if self.is_selected(row,col) {
                        if firstselec.is_none() {
                            firstselec = Some(col-xbound1);
                        }
                        lastselec = Some(col-xbound1);
                    }
                }
                let selected_text: Option<(usize, usize)> = firstselec.zip(lastselec);
                Self::render_line(row, &e.get_visible_graphemes(xbound1..xbound2), selected_text);
            } else  {
                Self::render_line(row, "~", None);
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
                Self::render_line(row, "~", None);
            }
            if row.saturating_add(1) < height {
                let _ = Terminal::move_caret_to(Coords {row: row+1, col: 0} );
            }
        }
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
        let _ = Terminal::print(&welcome_msg, None);
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
        Coords {row, col}
    }

     fn move_text_location(&mut self, direction: Direction) {
         let Size { height, .. } = self.size;
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
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
        let terminal_size = Terminal::size().unwrap_or_default();
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Size {
                width: terminal_size.width,
                height: terminal_size.height - 1,
            },
            selection: Selection::default(),
            text_location: Location::default(),
            scroll_offset: Coords::default(),
            bmode: Bmode::Normal,
        }
    }
}

impl Bmode {
    fn as_str(&self) -> String {
        match self {
            Bmode::Insert => "Insert".to_string(),
            Bmode::Normal => "Normal".to_string(),
            Bmode::Replace => "Replace".to_string(),
            Bmode::Visual => "Visual".to_string(),
        }
    }
}
