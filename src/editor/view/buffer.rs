use std::io::Error;
use super::line::Line;
use super::Location;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self, Error>{
        let file_contents = std::fs::read_to_string(filename)?;
        let mut lines = Vec::new();
        for line in file_contents.lines() {
            lines.push(Line::from(line));
        }
        Ok(Self {lines})
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
        if at.line_index > self.lines.len() {
            return;
        }
        if at.line_index == self.lines.len() {
            self.lines.push(Line::from(&character.to_string()));
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(character, at.grapheme_index);
        }
    }
}
