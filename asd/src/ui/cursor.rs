use crate::backend::display_char::EMOJI_LIST;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use ratatui::DefaultTerminal;
use std::io;

pub struct TerminalCursor {
    pub x: u16,
    pub y: u16,
    pub cursor_showing: bool,
    pub render_cursor_showing: bool,
}

impl TerminalCursor {
    pub(crate) fn set_to(&mut self, position: (u16, u16)) {
        self.x = position.0;
        self.y = position.1;
        self.show();
    }

    pub(crate) fn render1(&mut self) {
        if self.cursor_showing {
            execute!(
                io::stdout(),
                MoveTo(self.x, self.y)
            )
                .unwrap(); // todo: remove unwrap
        }
    }

    pub(crate) fn render2(&self, terminal: &mut DefaultTerminal) {
        if self.render_cursor_showing {
            if self.cursor_showing {
                terminal.show_cursor().unwrap();
            } else {
                terminal.hide_cursor().unwrap();
            }
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            x: u16::MAX,
            y: u16::MAX,
            cursor_showing: true,
            render_cursor_showing: true,
        }
    }

    pub(crate) fn hide(&mut self) {
        if self.cursor_showing {
            self.cursor_showing = false;
            self.render_cursor_showing = true;
        }
    }

    pub(crate) fn show(&mut self) {
        if !self.cursor_showing {
            self.cursor_showing = true;
            self.render_cursor_showing = true;
        }
    }

    pub(crate) fn render_emoji_queue(&mut self, emoji_queue: Vec<(u16, u16, u32)>) {
        for (x, y, emoji) in emoji_queue {
            execute!(
                io::stdout(),
                MoveTo(x, y),
            ).unwrap(); // todo: I don't think it will fail. Maybe: remove unwrap
            let lock = EMOJI_LIST.lock().unwrap();
            let str: &str = lock.get(emoji as usize & 0x3FFF_FFFF).unwrap();
            print!("{}", str);
        }
    }
}