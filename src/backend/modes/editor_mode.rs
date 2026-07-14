use crate::backend::event_handler::{EventFlags, EventHandler};
use crate::backend::modes::prompt_mode::SaveAsMode;
use crate::backend::modes::Mode;
use crate::edit_controller::EditController;
use crate::App;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::Frame;
use crate::backend::modes::menu_mode::MenuMode;

pub struct EditorMode {
    handler: EventHandler<()>,
}


impl Mode for EditorMode {
    fn handle_event(&mut self, app: &mut App, event: Event) {
        self.handler.handle_event(&mut (), app, event);
    }

    fn render_function(&mut self, _: &mut Frame) {}

    fn needs_terminal_cursor(&self) -> bool { false }
}

impl EditorMode {
    #[allow(unused_doc_comments)]
    pub(crate) fn new() -> EditorMode {
        EditorMode {
            handler: EventHandler::new(
                vec![
                    /// Placement Operator
                    |_, app, e, f| {
                        if (f & EventFlags::AllModifiers & (!EventFlags::M_SHIFT)) == EventFlags::empty() &&
                           let KeyCode::Char(ch) = e.code {
                            app.buffers.active_mut().place_char(ch, app.last_content_rect, &mut app.logs);
                            return false;
                        }
                        if (f & EventFlags::AllModifiers) == EventFlags::empty() &&
                            KeyCode::Enter == e.code {
                            app.buffers.active_mut().place_new_line(app.last_content_rect, &mut app.logs);
                            return false;
                        }
                        if (f & EventFlags::AllModifiers) == EventFlags::empty() &&
                            KeyCode::Tab == e.code {
                            app.buffers.active_mut().operate_tab(app.last_content_rect, &mut app.logs);
                            return false;
                        }
                        return true;
                    },


                    /// Shortcuts Operator
                    |_, app, e, f| {
                        if (f & EventFlags::AllModifiers) == EventFlags::empty() &&
                            KeyCode::Esc == e.code {
                            app.change_mode = Some(Box::new(MenuMode::new_menu_basic()));
                            return false;
                        }
                        if (f & EventFlags::AllModifiers) == EventFlags::M_CTRL | EventFlags::M_SHIFT &&
                            e.code == KeyCode::Char('z') {
                            app.buffers.active_mut().operate_redo(app.last_content_rect, &mut app.logs);
                            return false;
                        }
                        if (f & EventFlags::AllModifiers) == EventFlags::M_CTRL_ALT &&
                            e.code == KeyCode::Char('s') {
                            app.change_mode = Some(Box::new(
                                SaveAsMode::new(app)
                            ) as Box<dyn Mode>);
                            return false;
                        }

                        if (f & EventFlags::AllModifiers) == EventFlags::M_CTRL | EventFlags::M_ALT &&
                            e.code == KeyCode::Char('q') {
                            app.operate_force_quit();
                            return false;
                        }


                        if (f & EventFlags::AllModifiers) != EventFlags::M_CTRL {
                            return true;
                        }

                        match e.code {
                            KeyCode::Char('q') => {
                                app.operate_quit(); false
                            }
                            KeyCode::Char('f') => {
                                app.buffers.active_mut().scrollbar.freeze = !app.buffers.active().scrollbar.freeze; false
                            }
                            KeyCode::Char('s') => {
                                app.buffers.active_mut().operate_save(&mut app.logs); false
                            }
                            KeyCode::Char('c') => {
                                app.buffers.active_mut().operate_copy(app.last_content_rect, &mut app.logs); false
                            }
                            KeyCode::Char('v') => {
                                app.buffers.active_mut().operate_paste(app.last_content_rect, &mut app.logs); false
                            }
                            KeyCode::Char('x') => {
                                app.buffers.active_mut().operate_cut(app.last_content_rect, &mut app.logs); false
                            }
                            KeyCode::Char('z') => {
                                app.buffers.active_mut().operate_undo(app.last_content_rect, &mut app.logs); false
                            }
                            _ => true,
                        }
                    },


                    /// Arrow keys
                    |_, app, e, f| {
                        #[derive(Clone, Copy)]
                        enum Arrow {L, R, U, D, End, Home, PU, PD}
                        let arrow = match e.code {
                            KeyCode::Left => Arrow::L,
                            KeyCode::Right => Arrow::R,
                            KeyCode::Up => Arrow::U,
                            KeyCode::Down => Arrow::D,
                            KeyCode::Home => Arrow::Home,
                            KeyCode::End => Arrow::End,
                            KeyCode::PageUp => Arrow::PU,
                            KeyCode::PageDown => Arrow::PD,
                            _ => return true,
                        };

                        let active_buffer = app.buffers.active_mut();

                        if (f & EventFlags::AllModifiers) == EventFlags::M_ALT {
                            match arrow {
                                Arrow::L => active_buffer.operate_scroll_prev(1, &mut app.logs),
                                Arrow::R => active_buffer.operate_scroll_next(1, &mut app.logs),
                                Arrow::U => active_buffer.operate_scroll_prev_line(1, &mut app.logs),
                                Arrow::D => active_buffer.operate_scroll_next_line(1,  &mut app.logs),
                                Arrow::PU => active_buffer.operate_scroll_prev_line(app.last_content_rect.height as usize, &mut app.logs),
                                Arrow::PD => active_buffer.operate_scroll_next_line(app.last_content_rect.height as usize, &mut app.logs),
                                Arrow::Home |
                                Arrow::End => return true,
                            }
                            return false;
                        }
                        if (f & EventFlags::M_ALT_SUPER) != EventFlags::M_NOTHING {
                            return true;
                        }

                        active_buffer.operate_arrow_begin(&mut app.logs);
                        for caret in active_buffer.carets.carets.iter_mut() {
                            let mut pos = *caret.get_position();

                            if (f & EventFlags::M_SHIFT) == EventFlags::M_SHIFT {
                                if pos.selection.is_none() {
                                    pos.selection.set_to(&pos.cursor);
                                }
                            } else {
                                pos.selection.none = true;
                            }

                            match ((f & EventFlags::M_CTRL) == EventFlags::M_CTRL, arrow) {
                                (false, Arrow::L) => pos.cursor.prev(&active_buffer.content),
                                (false, Arrow::R) => pos.cursor.next(&active_buffer.content),
                                (_, Arrow::U) => pos.cursor.up(1, &active_buffer.content),
                                (_, Arrow::D) => pos.cursor.down(1, &active_buffer.content),
                                (false, Arrow::End) => pos.cursor.end(&active_buffer.content),
                                (false, Arrow::Home) => pos.cursor.home(),
                                (_, Arrow::PU) => pos.cursor.up(app.last_content_rect.height as usize, &active_buffer.content),
                                (_, Arrow::PD) => pos.cursor.down(app.last_content_rect.height as usize, &active_buffer.content),
                                (true, Arrow::L) => pos.cursor.prev_word(&active_buffer.content),
                                (true, Arrow::R) => pos.cursor.next_word(&active_buffer.content),
                                (true, Arrow::End) => pos.cursor.ctrl_end(&active_buffer.content),
                                (true, Arrow::Home) => pos.cursor.ctrl_home(),
                            }

                            caret.set_position(pos);
                            caret.merge_sel_pos();
                        }
                        active_buffer.operate_arrow_end(app.last_content_rect);

                        return false;
                    },

                    /// Remove methods
                    |_, app, e, _f| {
                        match (e.code, e.modifiers) {
                            (KeyCode::Backspace, KeyModifiers::NONE) => {
                                app.buffers.active_mut().operate_backspace(app.last_content_rect, &mut app.logs); false
                            }
                            (KeyCode::Delete, KeyModifiers::NONE) => {
                                app.buffers.active_mut().operate_delete(app.last_content_rect, &mut app.logs); false
                            }
                            _ => true,
                        }
                    }
                ],
                vec![
                    /// Double Click Handler
                    EventHandler::default_double_click_handler,


                    /// Scroll
                    |_, _, app, e, _f| {
                        match e.kind {
                            MouseEventKind::ScrollDown => {
                                if (e.modifiers & KeyModifiers::SHIFT) == KeyModifiers::SHIFT {
                                    app.buffers.active_mut().scrollbar.next(10);
                                } else {
                                    app.buffers.active_mut().scrollbar.next_line(5);
                                } false
                            }
                            MouseEventKind::ScrollUp => {
                                if (e.modifiers & KeyModifiers::SHIFT) == KeyModifiers::SHIFT {
                                    app.buffers.active_mut().scrollbar.prev(10);
                                } else {
                                    app.buffers.active_mut().scrollbar.prev_line(5);
                                } false
                            }
                            MouseEventKind::ScrollLeft => {
                                app.buffers.active_mut().scrollbar.prev(10); false
                            }
                            MouseEventKind::ScrollRight => {
                                app.buffers.active_mut().scrollbar.next(10); false
                            }
                            _ => true,
                        }
                    },

                    |_, _, app, e, _f| {
                        const SHIFT_ALT: KeyModifiers = KeyModifiers::SHIFT.union(KeyModifiers::ALT);

                        match (e.kind, e.modifiers) {
                            (MouseEventKind::Down(MouseButton::Left), KeyModifiers::NONE) |
                            (MouseEventKind::Down(MouseButton::Left), SHIFT_ALT) => {
                                app.buffers.active_mut().operate_single_click(e.column, e.row, app.last_content_rect, &mut app.logs); false
                            }
                            (MouseEventKind::Down(MouseButton::Left), KeyModifiers::CONTROL) => {
                                app.buffers.active_mut().operate_add_cursor(e.column, e.row, app.last_content_rect, &mut app.logs); false
                            }
                            (MouseEventKind::Drag(MouseButton::Left), KeyModifiers::NONE) => {
                                app.buffers.active_mut().operate_mouse_select(e.column, e.row, app.last_content_rect, &mut app.logs); false
                            }
                            (MouseEventKind::Drag(MouseButton::Left), SHIFT_ALT) => {
                                app.buffers.active_mut().operate_multicursor_select(e.column, e.row, app.last_content_rect, &mut app.logs); false
                            }
                            _ => true,
                        }
                    }
                ],
                vec![
                    |_, app, e, _f| {
                        app.buffers.active_mut().operate_double_click(e.column, e.row, app.last_content_rect, &mut app.logs); false
                    }
                ]
            ),
        }
    }
}