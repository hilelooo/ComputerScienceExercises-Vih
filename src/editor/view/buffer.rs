use std::io::{Write, Error};
use std::fs::{read_to_string, File};
use super::line::Line;
use super::Location;

pub struct Buffer {
    pub lines: Vec<Line>,
    pub filename: String,
    pub dirty: bool,
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self, Error>{
        let file_contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for line in file_contents.lines() {
            lines.push(Line::from(line));
        }
        Ok(Self {lines, filename: filename.to_string(), dirty: false})
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let filename = &self.filename {
            let mut file = File::create(filename)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
            self.dirty = false
        }
        Ok(())
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
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(character, at.grapheme_index);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count() && self.lines.len() > at.line_index + 1 {
                let next_line = self.lines.remove(at.line_index + 1);
                self.lines[at.line_index].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_index < line.grapheme_count() {
                self.lines[at.line_index].delete(at.grapheme_index);
                self.dirty = true;
                // normal case
            }
        }
    }

    pub fn insert_line(&mut self, at: Location) {
        if at.line_index == self.lines.len() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let newline = line.split(at.grapheme_index);
            self.lines.insert(at.line_index + 1, newline);
            self.dirty = true;
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: Vec::<Line>::default(),
            filename: "default.txt".to_string(),
            dirty: false,
        }
    }
}
