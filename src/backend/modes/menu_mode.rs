use crate::assets::colors::colors::{C_MENU_BG, C_MENU_FG};
use crate::backend::buffers::Buffers;
use crate::backend::event_handler::{EventFlags, EventHandler};
use crate::backend::modes::editor_mode::EditorMode;
use crate::backend::modes::Mode;
use crate::App;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{Block, Clear, Widget};
use ratatui::Frame;
use std::borrow::Cow;

pub struct MenuMode {
    event_handler: EventHandler<Vec<MenuCommand>>,
    commands: Vec<MenuCommand>,
}

pub struct MenuCommand {
    text: Cow<'static, str>,
    shortcut_symbol: Cow<'static, str>,
    handler: Option<(KeyCode, KeyModifiers, fn(&mut App, &KeyEvent, EventFlags))>,
}

impl MenuCommand {
    fn new(text: Cow<'static, str>, shortcut_symbol: Cow<'static, str>, handler: Option<(KeyCode, KeyModifiers, fn(&mut App, &KeyEvent, EventFlags))>) -> Self {
        Self {
            text,
            shortcut_symbol,
            handler
        }
    }
}

impl Mode for MenuMode {
    fn handle_event(&mut self, app: &mut App, event: Event) {
        self.event_handler.handle_event(&mut self.commands, app, event)
    }

    fn render_function(&mut self, frame: &mut Frame) {
        let needed_rows = 2 + self.commands.len() as u16;
        let shortcut_symbols_len = self.commands.iter().map(|c| c.shortcut_symbol.chars().count() as u16).max().unwrap();
        let commands_len = self.commands.iter().map(|c| c.text.chars().count() as u16).max().unwrap();
        let needed_cols = 5 +
            shortcut_symbols_len +
            commands_len;

        let area = frame.area();
        let hor_layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(needed_cols), Constraint::Fill(1)]);
        let [_, area, _] = area.layout(&hor_layout);
        let ver_layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(needed_rows), Constraint::Fill(1)]);
        let [_, area, _] = area.layout(&ver_layout);

        Clear::default()
            .render(area, frame.buffer_mut());
        Block::new()
            .fg(C_MENU_FG)
            .bg(C_MENU_BG)
            .render(area, frame.buffer_mut());

        for (i, c) in self.commands.iter().enumerate() {
            Text::from(c.shortcut_symbol.as_ref())
                .render(Rect {
                    x: area.x + 1,
                    y: area.y + 1 + i as u16,
                    width: shortcut_symbols_len,
                    height: 1,
                }, frame.buffer_mut());
            Text::from(c.text.as_ref())
                .alignment(Alignment::Right)
                .render(Rect {
                    x: area.x + 4 + shortcut_symbols_len,
                    y: area.y + 1 + i as u16,
                    width: commands_len,
                    height: 1,
                }, frame.buffer_mut());
        }
    }

    fn needs_terminal_cursor(&self) -> bool {
        true
    }
}

impl MenuMode {
    pub(crate) fn new(commands: Vec<MenuCommand>) -> MenuMode {
        Self {
            event_handler: EventHandler::new(
                vec![
                    |a, app, e, f| {
                        app.change_mode = Some(Box::new(EditorMode::new()));
                        for c in a {
                            if let Some((kc, m, handler)) = c.handler {
                                if kc == e.code && m == e.modifiers {
                                    handler(app, e, f);
                                    return false;
                                }
                            }
                        }
                        false
                    },
                ],
                vec![],
                vec![],
            ),
            commands
        }
    }

    pub(crate) fn new_menu_basic() -> MenuMode {
        let mut commands = Vec::new();
        commands.push(MenuCommand::new("Close the menu".into(), "esc".into(), None, ));
        commands.push(MenuCommand::new("Quit this buffer".into(), "q".into(),
                                       Some((KeyCode::Char('q'), KeyModifiers::NONE, Buffers::quit_current_evt))));
        commands.push(MenuCommand::new("Force quit this buffer".into(), "q".into(),
                                       Some((KeyCode::Char('Q'), KeyModifiers::SHIFT, Buffers::force_quit_current_evt))));
        commands.push(MenuCommand::new("Open help".into(), "h".into(),
                                       Some((KeyCode::Char('h'), KeyModifiers::NONE, Buffers::open_help_evt))));
        // commands.push(MenuCommand::new("Open file".into(), "h".into(),
        //                                Some((KeyCode::Char('o'), KeyModifiers::NONE, Buffers::open_help_evt))));
        Self::new(commands)
    }
}