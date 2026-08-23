use crate::App;
use crossterm::event::Event;
use ratatui::{Frame};

pub mod editor_mode;
pub mod prompt_mode;
mod menu_mode;

pub trait Mode {
    fn handle_event(&mut self, app: &mut App, event: Event);
    fn render_function(&mut self, frame: &mut Frame);
    fn needs_terminal_cursor(&self) -> bool;
}