use super::{
    terminal::{Size,Terminal},
    DocumentStatus,
};

pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    width: usize,
    position_y: usize,
}


impl StatusBar {
    pub fn new() -> Self {
        let size = Terminal::size().unwrap_or_default();
        Self {
            current_status: DocumentStatus::default(),
            needs_redraw: true,
            width: size.width,
            position_y: size.height - 1,
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        self.position_y = size.height - 1;
        self.needs_redraw = true
    }

    pub fn update_status(&mut self, status: DocumentStatus) {
        if status != self.current_status {
            self.current_status = status;
            self.needs_redraw = true;
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {return;}
        let line_idx = self.current_status.current_line_index + 1;
        let total_lines = self.current_status.total_lines + 1;
        let width = self.width / 3;
        let lines_info = format!("{line_idx}/{total_lines}");
        let mode = &self.current_status.bmode_string;
        let name = &self.current_status.filename;
        let fileinfo =  if self.current_status.is_modified {
             name.to_owned()+ "*"
        } else {
             name.to_string()
        };
        let status = format!("{:<width$}{:^width$}{:>width$}",mode,fileinfo,lines_info);
        let _ = Terminal::print_row(self.position_y, &status, None);
        self.needs_redraw = false;
    }
}
