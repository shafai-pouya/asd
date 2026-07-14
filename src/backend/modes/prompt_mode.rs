use crate::assets::colors::colors::{C_BG_CURSOR_SELECTION, C_BG_SELECTION, C_FG_CURSOR_SELECTION, C_FG_SELECTION, C_LOG_ERROR, C_LOG_INFO, C_LOG_TODO};
use crate::backend::event_handler::{EventFlags, EventHandler};
use crate::backend::modes::editor_mode::EditorMode;
use crate::backend::modes::Mode;
use crate::ui::log::Log;
use crate::App;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Widget};
use ratatui::Frame;
use std::path::Path;

pub struct SaveAsData {
    pub filepath: String,
    pub cursor: u16,
    pub selection: Option<u16>,
    pub last_content_rect: Rect
}

pub struct SaveAsMode {
    pub data: SaveAsData,
    pub my_logs_collected: Vec<Log>,
    event_handler: EventHandler<SaveAsData>,
}

impl Mode for SaveAsMode {
    fn handle_event(&mut self, app: &mut App, event: Event) {
        self.data.last_content_rect = app.last_content_rect;

        let prev_count = app.logs.len();
        self.event_handler.handle_event(&mut self.data, app, event);
        for i in prev_count..app.logs.len() {
            self.my_logs_collected.push(app.logs[i].clone())
        }

        if app.change_mode.is_some() {
            return;
        }

        app.logs.clear();
        for i in &self.my_logs_collected {
            app.logs.push(i.clone());
        }
        app.logs.push(Log {
            message: "Save to file:".to_string(),
            color: C_LOG_INFO,
        });
        app.logs.push(Log {
            message: self.data.filepath.clone(),
            color: C_LOG_INFO,
        });


        // Render cursor
        if let Some(_) = self.data.selection {
            app.terminal_cursor.hide();
        } else {
            app.terminal_cursor.set_to((self.data.cursor, app.last_content_rect.height + 1));
        }
    }

    fn render_function(&mut self, frame: &mut Frame) {
        if let Some(sel) = self.data.selection {
            let min = sel.min(self.data.cursor);
            let max = sel.max(self.data.cursor);

            Block::new()
                .bg(C_BG_SELECTION)
                .fg(C_FG_SELECTION)
                .render(Rect {
                    x: min,
                    y: self.data.last_content_rect.height + 1,
                    width: max - min + 1,
                    height: 1
                }, frame.buffer_mut());

            Block::new()
                .bg(C_BG_CURSOR_SELECTION)
                .fg(C_FG_CURSOR_SELECTION)
                .render(Rect {
                    x: sel,
                    y: self.data.last_content_rect.height + 1,
                    width: 1,
                    height: 1,
                }, frame.buffer_mut());
        }
    }

    fn needs_terminal_cursor(&self) -> bool {
        true
    }
}

impl SaveAsMode {
    #[allow(unused_doc_comments)]
    pub(crate) fn new(app: &mut App) -> Self {
        let mut self_ = Self {
            data: SaveAsData {
                filepath: String::new(),
                cursor: 0,
                selection: None,
                last_content_rect: Rect::default(),
            },
            my_logs_collected: Vec::new(),
            event_handler: EventHandler::new(
                vec![
                    /// Placement Operator
                     |data, app, e, f| {
                         if (f & EventFlags::AllModifiers & (!EventFlags::M_SHIFT)) == EventFlags::empty() &&
                             let KeyCode::Char(ch) = e.code {
                             if let Some(sel) = data.selection {
                                 let min = sel.min(data.cursor);
                                 let max = sel.max(data.cursor);
                                 data.filepath.drain(min as usize..max as usize);
                                 data.selection = None;
                                 data.cursor = min;
                             }
                             data.filepath.insert(data.cursor as usize, ch);
                             data.cursor += 1;
                             return false;
                         }
                         if (f & EventFlags::AllModifiers) == EventFlags::empty() &&
                             KeyCode::Enter == e.code {
                             app.change_mode = Some(Box::new(EditorMode::new()));
                             app.logs.clear();
                             let active_buffer = app.buffers.active_mut();
                             let _ = active_buffer.save(Some(Path::new(&data.filepath)), &mut app.logs);
                             active_buffer.modified = true;
                             return false;
                         }
                         if (f & EventFlags::AllModifiers) == EventFlags::empty() &&
                             KeyCode::Tab == e.code {
                             app.logs.push(Log {
                                 message: "Autocompletion is not implemented yet (todo)".to_string(),
                                 color: C_LOG_TODO,
                             });
                             // todo!();
                             return false;
                         }
                         return true;
                     },


                    /// Shortcuts Operator
                    |data, app, e, f| {
                        if (f & EventFlags::AllModifiers) == EventFlags::empty() &&
                            e.code == KeyCode::Esc {
                            app.change_mode = Some(Box::new(EditorMode::new()));
                            app.logs.clear();
                        }

                        match e.code {
                            KeyCode::Char('c') => {
                                if let Some(sel) = data.selection {
                                    let min = sel.min(data.cursor);
                                    let max = sel.max(data.cursor);
                                    App::op_set_clipboard(data.filepath[min as usize..max as usize].as_bytes(), &mut app.logs);
                                }
                                false
                            }
                            KeyCode::Char('v') => {
                                if let Some(clip) = App::op_get_clipboard(&mut app.logs) {
                                    if clip.contains('\n') {
                                        app.logs.push(Log {
                                            message: "There is enter in your pasting thingy".to_string(),
                                            color: C_LOG_ERROR,
                                        });
                                        return false;
                                    }
                                    let range = if let Some(sel) = data.selection {
                                        let min = sel.min(data.cursor);
                                        let max = sel.max(data.cursor);
                                        data.selection = None;
                                        data.cursor = min;
                                        min as usize..max as usize
                                    } else {
                                        data.cursor as usize..data.cursor as usize
                                    };
                                    data.filepath.replace_range(range, &clip);
                                    data.cursor += clip.len() as u16;
                                }
                                false
                            }
                            KeyCode::Char('x') => {
                                if let Some(sel) = data.selection {
                                    let min = sel.min(data.cursor);
                                    let max = sel.max(data.cursor);
                                    data.selection = None;
                                    data.cursor = min;
                                    App::op_set_clipboard(data.filepath[min as usize..max as usize].as_bytes(), &mut app.logs);
                                    data.filepath.drain(min as usize..max as usize);
                                }
                                false
                            }
                            _ => true,
                        }
                    },


                    /// Arrow keys
                    |data, _app, e, f| {
                        if (f & EventFlags::M_CTRL_ALT_SUPER) != EventFlags::M_NOTHING {
                            return true;
                        }
                        match (e.code, (f & EventFlags::M_SHIFT) != EventFlags::empty()) {
                            (KeyCode::Left, false) => {
                                data.selection = None;
                                data.cursor = data.cursor.saturating_sub(1);
                                false
                            },
                            (KeyCode::Right, false) => {
                                data.selection = None;
                                data.cursor += 1;
                                if data.cursor > data.filepath.len() as u16 {
                                    data.cursor = data.filepath.len() as u16;
                                }
                                false
                            },
                            (KeyCode::Left, true) => {
                                if data.selection.is_none() {
                                    data.selection = Some(data.cursor);
                                }
                                data.cursor = data.cursor.saturating_sub(1);
                                false
                            },
                            (KeyCode::Right, true) => {
                                if data.selection.is_none() {
                                    data.selection = Some(data.cursor);
                                }
                                data.cursor += 1;
                                if data.cursor > data.filepath.len() as u16 {
                                    data.cursor = data.filepath.len() as u16;
                                }
                                false
                            },
                            _ => true,
                        }
                    },

                    /// Remove methods
                    |data, _app, e, _f| {
                        match (e.code, e.modifiers) {
                            (KeyCode::Backspace, KeyModifiers::NONE) => {
                                if let Some(sel) = data.selection {
                                    let min = sel.min(data.cursor);
                                    let max = sel.max(data.cursor);
                                    data.selection = None;
                                    data.cursor = min;
                                    data.filepath.drain(min as usize..max as usize);
                                } else {
                                    if data.cursor != 0 {
                                        data.cursor -= 1;
                                        data.filepath.remove(data.cursor as usize);
                                    }
                                }
                                false
                            }
                            (KeyCode::Delete, KeyModifiers::NONE) => {
                                if let Some(sel) = data.selection {
                                    let min = sel.min(data.cursor);
                                    let max = sel.max(data.cursor);
                                    data.selection = None;
                                    data.cursor = min;
                                    data.filepath.drain(min as usize..max as usize);
                                } else {
                                    if data.cursor < data.filepath.len() as u16 {
                                        data.filepath.remove(data.cursor as usize);
                                    }
                                }
                                false
                            }
                            _ => true,
                        }
                    }
                ],
                vec![
                    /// Double Click Handler
                    EventHandler::default_double_click_handler,

                    |data, _, _app, e, _f| {
                        match (e.kind, e.modifiers) {
                            (MouseEventKind::Down(MouseButton::Left), KeyModifiers::NONE) => {
                                data.selection = None;
                                data.cursor = e.column;
                                if data.cursor > data.filepath.len() as u16 {
                                    data.cursor = data.filepath.len() as u16;
                                }
                                false
                            }
                            (MouseEventKind::Drag(MouseButton::Left), KeyModifiers::NONE) => {
                                if data.selection.is_none() {
                                    data.selection = Some(data.cursor);
                                }
                                data.cursor = e.column;
                                if data.cursor > data.filepath.len() as u16 {
                                    data.cursor = data.filepath.len() as u16;
                                }
                                false
                            }
                            _ => true,
                        }
                    }
                ],
                vec![
                    |_, app, _e, _f| {
                        // todo!();
                        app.logs.push(Log {
                            message: "Double clicking when saving as is not implemented yet (todo)".to_string(),
                            color: C_LOG_TODO,
                        });
                        false
                    }
                ]
            )
        };
        self_.handle_event(app, Event::Resize(0, 0));
        self_
    }
}