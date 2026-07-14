use ratatui::{DefaultTerminal, Frame};

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

    pub(crate) fn render1(&mut self, frame: &mut Frame) {
        if self.cursor_showing {
            frame.set_cursor_position((self.x, self.y));
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
            render_cursor_showing: false,
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
}