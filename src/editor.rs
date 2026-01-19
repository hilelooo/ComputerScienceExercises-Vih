use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    io::Error,
    panic::{set_hook,take_hook},};
mod terminal;
mod view;
mod editorcommand;

use view::View;
use terminal::Terminal;


pub struct Editor {
    should_quit:  bool,
    view: View,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move|panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut view = View::default();
        let args: Vec<String> = std::env::args().collect();
        if let Some(filename) = args.get(1) {
            view.load(filename);
        }
        Ok(Self {
            should_quit: false,
            view
        })
    }

    pub fn run(&mut self){
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("could not read event: {err:?}");
                    }
                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent {kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_,_) => true,
            _ => false,
        };

        if should_process {
            if self.view.handle_command(event) {
                self.should_quit = true;
            }
        }
    }

    fn refresh_screen(&mut self){
        let _ = Terminal::hide_caret();
        self.view.render();
        let _ = Terminal::move_caret_to(self.view.caret_position());
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye\r\n");
        }
    }
}
