use crate::assets::colors::colors::C_LOG_INFO;
use crate::backend::buffer::Buffer;
use crate::backend::caret::{Caret, CursorEditor, Position};
use crate::backend::content::whitespaces_in_the_start_of_the_line;
use crate::backend::cursor::Cursor;
use crate::backend::display_char::DisplayChar;
use crate::backend::display_string::DisplayString;
use crate::backend::encoding::Encoding;
use crate::backend::little_string::{LittleString, LittleStringUni};
use crate::backend::selection::Selection;
use crate::edit_operators::EditOperators;
use crate::ui::log::{LOGS, Log};
use crate::{Clipboard, movec};
use ratatui::layout::Rect;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;

/// This trait made to be implemented for only one struct. It made
/// to implement some functions in another file, for readability,
/// and also use self with it
pub trait EditController {
    fn place_char(&mut self, ch: char, last_content_rect: Rect);
    fn place_new_line(&mut self, last_content_rect: Rect);
    fn operate_backspace(&mut self, last_content_rect: Rect);
    fn operate_delete(&mut self, last_content_rect: Rect);
    fn operate_tab(&mut self, last_content_rect: Rect);
    fn operate_scroll_prev(&mut self, i: u16);
    fn operate_scroll_next_line(&mut self, i: usize);
    fn operate_scroll_prev_line(&mut self, i: usize);
    fn operate_scroll_next(&mut self, i: u16);
    fn operate_arrow_begin(&mut self);
    fn operate_arrow_end(&mut self, last_content_rect: Rect);
    fn operate_undo(&mut self, last_content_rect: Rect);
    fn operate_redo(&mut self, last_content_rect: Rect);
    fn operate_copy(&mut self, last_content_rect: Rect, clipboard: &mut Clipboard);
    fn operate_cut(&mut self, last_content_rect: Rect, clipboard: &mut Clipboard);
    fn operate_paste(&mut self, last_content_rect: Rect, clipboard: &mut Clipboard);
    fn operate_save(&mut self);

    fn operate_single_click(&mut self, col: u16, row: u16, last_content_rect: Rect);
    fn operate_add_cursor(&mut self, col: u16, row: u16, last_content_rect: Rect);
    fn operate_mouse_select(&mut self, col: u16, row: u16, last_content_rect: Rect);
    fn operate_multicursor_select(&mut self, col: u16, row: u16, last_content_rect: Rect);
    fn operate_double_click(&mut self, col: u16, row: u16, last_content_rect: Rect);
}

impl EditController for Buffer {
    /// Safety: Make sure the char matches the current encoding
    fn place_char(&mut self, ch: char, last_content_rect: Rect) {
        LOGS.clear();
        self.op_materialize_virtual_spaces();
        for caret_idx in 0..self.carets.carets.len() {
            let mut cursor_editor = CursorEditor {
                cursors: &mut self.carets,
                cursor: caret_idx,
            };
            unsafe {
                let ls = match self.encoding {
                    Encoding::UTF8(_) => {
                        let min = cursor_editor.cursors.carets[cursor_editor.cursor]
                            .get_position()
                            .get_min();
                        if min.1 == 0 {
                            let mut s = DisplayString::empty();
                            DisplayChar::from_utf8_grapheme_to_dstring(
                                ch.encode_utf8(&mut [0; 4]),
                                &mut s,
                            );
                            LittleString::Big(s)
                        } else {
                            struct S(String);
                            impl Write for S {
                                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                                    self.0.push_str(str::from_utf8(buf).unwrap());
                                    Ok(buf.len())
                                }

                                fn flush(&mut self) -> std::io::Result<()> {
                                    Ok(())
                                }
                            }
                            let mut s = S(String::new());
                            self.content[min.0][min.1 - 1]
                                .utf8__write_to(&mut s)
                                .unwrap();
                            s.0.push(ch);
                            let mut g = s.0.graphemes(true);
                            let _ = g.next();
                            if g.next().is_some() {
                                LittleString::from_one_cell_utf8_char_unchecked(ch)
                            } else {
                                let lsu = LittleStringUni::new(&s.0);
                                let mut ls = LittleString::empty();
                                ls.push(DisplayChar::from_lsu(lsu));
                                cursor_editor.cursors.carets[cursor_editor.cursor]
                                    .get_position_mut_unchecked()
                                    .set_min((min.0, min.1 - 1));
                                ls
                            }
                        }
                    }
                    Encoding::Raw => LittleString::from_raw(ch.encode_utf8(&mut [0; 4]).as_bytes()),
                };
                self.content
                    .replace_text(&mut self.checkpoints, &mut cursor_editor, movec!(ls))
            } // todo: maybe it isn't one cell or doesn't match the encoding
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn place_new_line(&mut self, last_content_rect: Rect) {
        LOGS.clear();
        self.op_no_virtual_spaces();
        for caret_idx in 0..self.carets.carets.len() {
            let ws = whitespaces_in_the_start_of_the_line(
                &self.content[self.carets.carets[caret_idx].get_position().get_min().0],
            );
            let mut cursor_editor = CursorEditor {
                cursors: &mut self.carets,
                cursor: caret_idx,
            };
            unsafe {
                self.content.replace_text(
                    &mut self.checkpoints,
                    &mut cursor_editor,
                    movec!(LittleString::empty(), LittleString::from_slice(ws)),
                )
            } // Safety: spaces matches all encodings
        }
        self.commit();
        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_backspace(&mut self, last_content_rect: Rect) {
        LOGS.clear();
        self.op_no_virtual_spaces();
        if self.carets.any_selected() {
            for caret_idx in 0..self.carets.carets.len() {
                let mut cursor_editor = CursorEditor {
                    cursors: &mut self.carets,
                    cursor: caret_idx,
                };
                unsafe {
                    self.content
                        .replace_text(&mut self.checkpoints, &mut cursor_editor, movec!())
                } // Safety: You all placing nothing and don't have to worry about the encoding
            }
            self.commit();
        } else {
            let commit = self
                .carets
                .carets
                .iter()
                .any(|c| c.get_position().cursor().col == 0);
            for caret_idx in 0..self.carets.carets.len() {
                self.carets.carets[caret_idx].selection_make_backwards(&self.content);
                let mut ce = CursorEditor {
                    cursors: &mut self.carets,
                    cursor: caret_idx,
                };
                unsafe {
                    self.content
                        .replace_text(&mut self.checkpoints, &mut ce, movec!());
                } // Safety: You all placing nothing and don't have to worry about the encoding
            }
            if commit {
                self.commit();
            }
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_delete(&mut self, last_content_rect: Rect) {
        LOGS.clear();
        self.op_no_virtual_spaces();
        if self.carets.any_selected() {
            for caret_idx in 0..self.carets.carets.len() {
                let mut cursor_editor = CursorEditor {
                    cursors: &mut self.carets,
                    cursor: caret_idx,
                };
                unsafe {
                    self.content
                        .replace_text(&mut self.checkpoints, &mut cursor_editor, movec!())
                } // Safety: You all placing nothing and don't have to worry about the encoding
            }
            self.commit();
        } else {
            let commit = self.carets.carets.iter().any(|c| {
                c.get_position().cursor().col == self.content[c.get_position().cursor().line].len()
            });
            for caret_idx in 0..self.carets.carets.len() {
                self.carets.carets[caret_idx].selection_make_forwards(&self.content);
                let mut ce = CursorEditor {
                    cursors: &mut self.carets,
                    cursor: caret_idx,
                };
                unsafe {
                    self.content
                        .replace_text(&mut self.checkpoints, &mut ce, movec!());
                } // Safety: You all placing nothing and don't have to worry about the encoding
            }
            if commit {
                self.commit();
            }
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_tab(&mut self, last_content_rect: Rect) {
        // todo: unhandled selection
        LOGS.clear();
        self.op_materialize_virtual_spaces();
        for caret_idx in 0..self.carets.carets.len() {
            let mut cursor_editor = CursorEditor {
                cursors: &mut self.carets,
                cursor: caret_idx,
            };
            let tab_string = Self::op_get_tab_little_string(&cursor_editor, self.tab_size);
            unsafe {
                self.content.replace_text(
                    &mut self.checkpoints,
                    &mut cursor_editor,
                    movec!(tab_string),
                );
            } // spaces matches the all encodings
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_scroll_prev(&mut self, i: u16) {
        LOGS.clear();
        self.scrollbar.prev(i);
    }
    fn operate_scroll_next_line(&mut self, i: usize) {
        LOGS.clear();
        self.scrollbar.next_line(i);
    }
    fn operate_scroll_prev_line(&mut self, i: usize) {
        LOGS.clear();
        self.scrollbar.prev_line(i);
    }
    fn operate_scroll_next(&mut self, i: u16) {
        LOGS.clear();
        self.scrollbar.next(i);
    }
    fn operate_arrow_begin(&mut self) {
        LOGS.clear();
        self.commit();
    }
    fn operate_arrow_end(&mut self, last_content_rect: Rect) {
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_undo(&mut self, last_content_rect: Rect) {
        self.commit();
        if self.checkpoints.cursor_lened == 0 {
            LOGS.push(Log {
                message: "[I] No more checkpoints available".to_string(),
                color: C_LOG_INFO,
                handler: None,
            });
            return;
        }
        LOGS.clear();

        self.checkpoints.cursor_lened -= 1;
        let checkpoint = &self.checkpoints.others[self.checkpoints.cursor_lened];

        self.carets
            .carets
            .resize_with(checkpoint.inner.len(), || Caret::new());
        for (idx, edit) in checkpoint.inner.iter().enumerate().rev() {
            unsafe {
                self.carets.carets[idx].set_position_unchecked(Position::new(
                    Cursor::new(edit.edit.start_line, edit.edit.start_col),
                    Selection::new(
                        edit.edit.added_data.len().saturating_sub(1) + edit.edit.start_line,
                        edit.edit
                            .added_data
                            .iter()
                            .skip(1)
                            .last()
                            .map(LittleString::len)
                            .unwrap_or(
                                edit.edit
                                    .added_data
                                    .get(0)
                                    .map(LittleString::len)
                                    .unwrap_or(0)
                                    + edit.edit.start_col,
                            ),
                    ),
                ));
            } // Safety: It WAS a valid location
            let mut ce = CursorEditor {
                cursor: idx,
                cursors: &mut self.carets,
            };
            unsafe {
                self.content
                    .replace_text_without_checkpoints(&mut ce, edit.edit.removed_data.clone())
            } // Safety: It WAS a text with the valid encoding
        }

        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_redo(&mut self, last_content_rect: Rect) {
        self.commit();
        if self.checkpoints.cursor_lened == self.checkpoints.others.len() {
            LOGS.push(Log {
                message: "[I] No more checkpoints available".to_string(),
                color: C_LOG_INFO,
                handler: None,
            });
            return;
        }
        LOGS.clear();

        let checkpoint = &self.checkpoints.others[self.checkpoints.cursor_lened];
        self.checkpoints.cursor_lened += 1;

        self.carets
            .carets
            .resize_with(checkpoint.inner.len(), || Caret::new());
        for (idx, edit) in checkpoint.inner.iter().enumerate().rev() {
            unsafe {
                self.carets.carets[idx].set_position_unchecked(Position::new(
                    Cursor::new(edit.edit.start_line, edit.edit.start_col),
                    Selection::new(
                        edit.edit.removed_data.len().saturating_sub(1) + edit.edit.start_line,
                        edit.edit
                            .removed_data
                            .iter()
                            .skip(1)
                            .last()
                            .map(LittleString::len)
                            .unwrap_or(
                                edit.edit
                                    .removed_data
                                    .get(0)
                                    .map(LittleString::len)
                                    .unwrap_or(0)
                                    + edit.edit.start_col,
                            ),
                    ),
                ));
            } // Safety: It WAS a valid position
            let mut ce = CursorEditor {
                cursor: idx,
                cursors: &mut self.carets,
            };
            unsafe {
                self.content
                    .replace_text_without_checkpoints(&mut ce, edit.edit.added_data.clone())
            } // Safety: It WAS a text with the valid encoding
        }

        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_copy(&mut self, last_content_rect: Rect, clipboard: &mut Clipboard) {
        LOGS.clear();
        self.op_no_virtual_spaces();
        self.commit();

        self.op_copy(clipboard);

        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_cut(&mut self, last_content_rect: Rect, clipboard: &mut Clipboard) {
        LOGS.clear();
        self.op_no_virtual_spaces();

        self.commit();
        self.op_copy(clipboard);
        for caret_idx in 0..self.carets.carets.len() {
            let mut cursor_editor = CursorEditor {
                cursors: &mut self.carets,
                cursor: caret_idx,
            };
            unsafe {
                self.content
                    .replace_text(&mut self.checkpoints, &mut cursor_editor, movec!())
            } // Safety: You all placing nothing and don't have to worry about the encoding
        }
        self.commit();

        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_paste(&mut self, last_content_rect: Rect, clipboard: &mut Clipboard) {
        LOGS.clear();

        self.commit();
        self.op_materialize_virtual_spaces();

        let Some(clipboard) = self.op_get_each_cursor_clipboard(clipboard) else {
            return;
        };
        clipboard.1.into_map_enumerate(|(idx, to_place)| {
            let mut ce = CursorEditor {
                cursor: idx,
                cursors: &mut self.carets,
            };
            unsafe {
                self.content.replace_text(
                    &mut self.checkpoints,
                    &mut ce,
                    to_place.map(|a| a.encoding_change(clipboard.0, self.encoding)),
                );
            }
        });

        self.commit();

        self.buffer_modified();
        self.carets.merge();
        self.carets
            .ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_save(&mut self) {
        self.commit();
        self.save(None);
    }

    fn operate_single_click(&mut self, col: u16, row: u16, last_content_rect: Rect) {
        LOGS.clear();
        self.commit();

        let x = (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize;
        let y = (row.wrapping_sub(last_content_rect.y)) as usize + self.scrollbar.top_position;
        self.carets.set_cursor_mouse(x, y, &self.content);
        self.drag_start_pos = (x, y);
    }

    fn operate_add_cursor(&mut self, col: u16, row: u16, last_content_rect: Rect) {
        LOGS.clear();
        self.commit();

        self.carets.add_cursor(
            (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize,
            row.wrapping_sub(last_content_rect.y) as usize + self.scrollbar.top_position,
            &self.content,
        );
    }

    fn operate_mouse_select(&mut self, col: u16, row: u16, last_content_rect: Rect) {
        LOGS.clear();
        self.commit();

        let cert_ptr = &mut self.carets.carets[0];
        if cert_ptr.is_selection_none() {
            cert_ptr.set_selection_to_cursor();
        }
        cert_ptr.set_just_cursor_mouse(
            (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize,
            row.wrapping_sub(last_content_rect.y) as usize + self.scrollbar.top_position,
            &self.content,
        );
    }

    fn operate_multicursor_select(&mut self, col: u16, row: u16, last_content_rect: Rect) {
        LOGS.clear();
        self.commit();

        let x = (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize;
        let y = row.wrapping_sub(last_content_rect.y) as usize + self.scrollbar.top_position;
        let start_line = y.min(self.drag_start_pos.1).min(self.content.len() - 1);
        let end_line = y.max(self.drag_start_pos.1).min(self.content.len() - 1);
        let n_lines = 1 + end_line - start_line;
        self.carets.carets.resize_with(n_lines, Caret::new);
        let mut idx = 0;
        for l in start_line..=end_line {
            let mut cursor = Cursor::new(l, x);
            cursor.validate_wide_chars(&self.content);
            let mut selection = Selection::new(l, self.drag_start_pos.0);
            selection.validate_wide_chars(&self.content);
            unsafe {
                self.carets.carets[idx].set_position_unchecked(Position::new(cursor, selection));
            }
            self.carets.carets[idx].merge_sel_pos();
            idx += 1;
        }
    }

    fn operate_double_click(&mut self, col: u16, row: u16, last_content_rect: Rect) {
        LOGS.clear();
        self.commit();

        let x = (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize;
        let y = (row.wrapping_sub(last_content_rect.y)) as usize + self.scrollbar.top_position;

        let (line, mut start_col) = Cursor::clamp_position(x, y, &self.content);

        let current_char = self.content[line]
            .get(start_col)
            .map(|a| *a)
            .unwrap_or(unsafe { const { DisplayChar::from_one_cell_utf8_char_unchecked('_') } });
        if !current_char.is_variable_name() {
            return;
        }

        let mut end_col = start_col;

        while start_col != 0 {
            let char = self.content[line][start_col - 1]; // start col is valid, non-zero, and we're using the prev char of it. So, unwrap is ok
            if char.is_variable_name() {
                start_col -= 1;
            } else {
                break;
            }
        }

        let line_len = self.content[line].len();
        while end_col < line_len {
            let char = self.content[line][end_col]; // end_col is less than line_len, so unwrap is ok
            if char.is_variable_name() {
                end_col += 1;
            } else {
                break;
            }
        }

        // Double click occurs after a single click, so, there should be one cursor
        unsafe {
            self.carets.carets[0].set_position_unchecked(Position::new(
                Cursor::new(line, end_col),
                Selection::new(line, start_col),
            ));
        }
    }
}
