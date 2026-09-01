use crate::backend::buffers::BUFFERS;
use crate::backend::event_handler::EventHandler;
use crate::backend::modes::menu_mode::MenuMode;
use crate::backend::modes::prompt_mode::SaveAsMode;
use crate::backend::modes::Mode;
use crate::edit_controller::EditController;
use crate::edit_operators::EditOperators;
use crate::App;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::Frame;

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
                    |_, app, e| {
                        if (e.modifiers & (!KeyModifiers::SHIFT)) == KeyModifiers::empty() &&
                            let KeyCode::Char(ch) = e.code {
                            BUFFERS.get_change_guard().inner_mut().active_mut().place_char(ch, app.last_content_rect);
                            return false;
                        }
                        if e.modifiers == KeyModifiers::empty() &&
                            KeyCode::Enter == e.code {
                            BUFFERS.get_change_guard().inner_mut().active_mut().place_new_line(app.last_content_rect);
                            return false;
                        }
                        if e.modifiers == KeyModifiers::empty() &&
                            KeyCode::Tab == e.code {
                            BUFFERS.get_change_guard().inner_mut().active_mut().operate_tab(app.last_content_rect);
                            return false;
                        }
                        return true;
                    },


                    /// Shortcuts Operator
                    |_, app, e| {
                        if e.modifiers == KeyModifiers::empty() &&
                            KeyCode::Esc == e.code {
                            app.change_mode = Some(Box::new(MenuMode::new_menu_basic()));
                            return false;
                        }
                        if e.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) &&
                            e.code == KeyCode::Char('z') {
                            BUFFERS.get_change_guard().inner_mut().active_mut().operate_redo(app.last_content_rect);
                            return false;
                        }
                        if e.modifiers == (KeyModifiers::CONTROL | KeyModifiers::ALT) &&
                            e.code == KeyCode::Char('s') {
                            app.change_mode = Some(Box::new(
                                SaveAsMode::new(app)
                            ) as Box<dyn Mode>);
                            return false;
                        }

                        if e.modifiers == (KeyModifiers::CONTROL | KeyModifiers::ALT) &&
                            e.code == KeyCode::Char('q') {
                            app.operate_force_quit();
                            return false;
                        }


                        if e.modifiers != KeyModifiers::CONTROL {
                            return true;
                        }

                        match e.code {
                            KeyCode::Char('q') => {
                                app.operate_quit(); false
                            }
                            KeyCode::Char('f') => {
                                let mut buffers = BUFFERS.get_change_guard();
                                let active_buffer = buffers.inner_mut().active_mut();
                                active_buffer.scrollbar.freeze = !active_buffer.scrollbar.freeze; false
                            }
                            KeyCode::Char('s') => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_save(); false
                            }
                            KeyCode::Char('c') => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_copy(app.last_content_rect, &mut app.internal_clipboard); false
                            }
                            KeyCode::Char('v') => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_paste(app.last_content_rect, &mut app.internal_clipboard); false
                            }
                            KeyCode::Char('x') => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_cut(app.last_content_rect, &mut app.internal_clipboard); false
                            }
                            KeyCode::Char('z') => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_undo(app.last_content_rect); false
                            }
                            // KeyCode::Char('n') => {
                            //     let b = app.buffers.active_mut();
                            //     unsafe {
                            //         b.content.replace_text(&mut b.checkpoints,
                            //                                &mut CursorEditor { cursors: &mut b.carets, cursor: 0 },
                            //                                movec!(LittleString::Big(DisplayString::null())));
                            //     }
                            //     false
                            // }
                            _ => true,
                        }
                    },


                    /// Arrow keys
                    |_, app, e| {
                        #[derive(Clone, Copy, Eq, PartialEq)]
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

                        let mut buffers = BUFFERS.get_change_guard();
                        let active_buffer = buffers.inner_mut().active_mut();

                        if e.modifiers == KeyModifiers::ALT {
                            match arrow {
                                Arrow::L => active_buffer.operate_scroll_prev(1),
                                Arrow::R => active_buffer.operate_scroll_next(1),
                                Arrow::U => active_buffer.operate_scroll_prev_line(1),
                                Arrow::D => active_buffer.operate_scroll_next_line(1),
                                Arrow::PU => active_buffer.operate_scroll_prev_line(app.last_content_rect.height as usize),
                                Arrow::PD => active_buffer.operate_scroll_next_line(app.last_content_rect.height as usize),
                                Arrow::Home |
                                Arrow::End => return true,
                            }
                            return false;
                        }
                        if (e.modifiers & (KeyModifiers::ALT | KeyModifiers::SUPER)) != KeyModifiers::empty() {
                            return true;
                        }
                        
                        if (e.modifiers & KeyModifiers::CONTROL) == KeyModifiers::CONTROL && (arrow == Arrow::L || arrow == Arrow::R) {
                            active_buffer.op_no_virtual_spaces();
                        }

                        active_buffer.operate_arrow_begin();
                        for caret in active_buffer.carets.carets.iter_mut() {
                            let mut pos = *caret.get_position();

                            if (e.modifiers & KeyModifiers::SHIFT) != KeyModifiers::empty() {
                                if pos.is_selection_none() {
                                    pos.set_selection_to_cursor();
                                }
                            } else {
                                pos.set_selection_none();
                            }

                            match ((e.modifiers & KeyModifiers::CONTROL) == KeyModifiers::CONTROL, arrow) {
                                (false, Arrow::L) => pos.cursor_prev(&active_buffer.content),
                                (false, Arrow::R) => pos.cursor_next(&active_buffer.content),
                                (_, Arrow::U) => pos.cursor_up(1, &active_buffer.content),
                                (_, Arrow::D) => pos.cursor_down(1, &active_buffer.content),
                                (false, Arrow::End) => pos.cursor_end(&active_buffer.content),
                                (false, Arrow::Home) => pos.cursor_home(),
                                (_, Arrow::PU) => pos.cursor_up(app.last_content_rect.height as usize, &active_buffer.content),
                                (_, Arrow::PD) => pos.cursor_down(app.last_content_rect.height as usize, &active_buffer.content),
                                (true, Arrow::L) => { pos.cursor_prev_word(&active_buffer.content) },
                                (true, Arrow::R) => { pos.cursor_next_word(&active_buffer.content) },
                                (true, Arrow::End) => pos.cursor_ctrl_end(&active_buffer.content),
                                (true, Arrow::Home) => pos.cursor_ctrl_home(),
                            }

                            unsafe { caret.set_position_unchecked(pos); }
                            caret.merge_sel_pos();
                        }
                        active_buffer.operate_arrow_end(app.last_content_rect);

                        return false;
                    },

                    /// Remove methods
                    |_, app, e| {
                        match (e.code, e.modifiers) {
                            (KeyCode::Backspace, KeyModifiers::NONE) => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_backspace(app.last_content_rect); false
                            }
                            (KeyCode::Delete, KeyModifiers::NONE) => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_delete(app.last_content_rect); false
                            }
                            _ => true,
                        }
                    }
                ],
                vec![
                    /// Double Click Handler
                    EventHandler::default_double_click_handler,


                    /// Scroll
                    |_, _, _, e| {
                        match e.kind {
                            MouseEventKind::ScrollDown => {
                                if (e.modifiers & KeyModifiers::SHIFT) == KeyModifiers::SHIFT {
                                    BUFFERS.get_change_guard().inner_mut().active_mut().scrollbar.next(10);
                                } else {
                                    BUFFERS.get_change_guard().inner_mut().active_mut().scrollbar.next_line(5);
                                } false
                            }
                            MouseEventKind::ScrollUp => {
                                if (e.modifiers & KeyModifiers::SHIFT) == KeyModifiers::SHIFT {
                                    BUFFERS.get_change_guard().inner_mut().active_mut().scrollbar.prev(10);
                                } else {
                                    BUFFERS.get_change_guard().inner_mut().active_mut().scrollbar.prev_line(5);
                                } false
                            }
                            MouseEventKind::ScrollLeft => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().scrollbar.prev(10); false
                            }
                            MouseEventKind::ScrollRight => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().scrollbar.next(10); false
                            }
                            _ => true,
                        }
                    },

                    |_, _, app, e| {
                        const SHIFT_ALT: KeyModifiers = KeyModifiers::SHIFT.union(KeyModifiers::ALT);

                        match (e.kind, e.modifiers) {
                            (MouseEventKind::Down(MouseButton::Left), KeyModifiers::NONE) |
                            (MouseEventKind::Down(MouseButton::Left), SHIFT_ALT) => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_single_click(e.column, e.row, app.last_content_rect); false
                            }
                            (MouseEventKind::Down(MouseButton::Left), KeyModifiers::CONTROL) => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_add_cursor(e.column, e.row, app.last_content_rect); false
                            }
                            (MouseEventKind::Drag(MouseButton::Left), KeyModifiers::NONE) => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_mouse_select(e.column, e.row, app.last_content_rect); false
                            }
                            (MouseEventKind::Drag(MouseButton::Left), SHIFT_ALT) => {
                                BUFFERS.get_change_guard().inner_mut().active_mut().operate_multicursor_select(e.column, e.row, app.last_content_rect); false
                            }
                            _ => true,
                        }
                    }
                ],
                vec![
                    |_, app, e| {
                        BUFFERS.get_change_guard().inner_mut().active_mut().operate_double_click(e.column, e.row, app.last_content_rect); false
                    }
                ]
            ),
        }
    }
}