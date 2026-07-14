use crate::assets::colors::colors::C_INFO;
use crate::backend::buffer::Buffer;
use crate::backend::caret::{Caret, CursorEditor, Position};
use crate::backend::content::whitespaces_in_the_start_of_the_line;
use crate::backend::cursor::Cursor;
use crate::backend::little_string::LittleString;
use crate::backend::selection::Selection;
use crate::edit_operators::EditOperators;
use crate::ui::log::Log;
use crate::movec;
use ratatui::layout::Rect;

/// This trait made to be implemented for only one struct. It made
/// to implement some functions in another file, for readability, 
/// and also use self with it
pub trait EditController {
    fn place_char(&mut self, ch: char, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn place_new_line(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_backspace(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_delete(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_tab(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    // fn operate_quit(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    // fn operate_force_quit(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_scroll_prev(&mut self, i: u16, logs: &mut Vec<Log>);
    fn operate_scroll_next_line(&mut self, i: usize, logs: &mut Vec<Log>);
    fn operate_scroll_prev_line(&mut self, i: usize, logs: &mut Vec<Log>);
    fn operate_scroll_next(&mut self, i: u16, logs: &mut Vec<Log>);
    fn operate_arrow_begin(&mut self, logs: &mut Vec<Log>);
    fn operate_arrow_end(&mut self, last_content_rect: Rect);
    fn operate_undo(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_redo(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_copy(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_cut(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_paste(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_save(&mut self, logs: &mut Vec<Log>);

    fn operate_single_click(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_add_cursor(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_mouse_select(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_multicursor_select(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>);
    fn operate_double_click(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>);
}

impl EditController for Buffer {
    fn place_char(&mut self, ch: char, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.op_materialize_virtual_spaces();
        for caret_idx in 0..self.carets.carets.len() {
            let mut cursor_editor = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
            self.content.replace_text(&mut self.checkpoints, &mut cursor_editor, &movec!(ch.into()))
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn place_new_line(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.op_no_virtual_spaces();
        for caret_idx in 0..self.carets.carets.len() {
            let ws = whitespaces_in_the_start_of_the_line(&self.content[self.carets.carets[caret_idx].get_position().get_min().0]);
            let mut cursor_editor = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
            self.content.replace_text(&mut self.checkpoints, &mut cursor_editor, &movec!("".into(), ws.into()))
        }
        self.commit(logs);
        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }


    fn operate_backspace(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.op_no_virtual_spaces();
        if self.carets.any_selected() {
            for caret_idx in 0..self.carets.carets.len() {
                let mut cursor_editor = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
                self.content.replace_text(&mut self.checkpoints, &mut cursor_editor, &movec!())
            }
            self.commit(logs);
        } else {
            let commit = self.carets.carets.iter().any(|c| c.get_position().cursor.col == 0);
            for caret_idx in 0..self.carets.carets.len() {
                self.carets.carets[caret_idx].selection_make_backwards(&self.content);
                let mut ce = CursorEditor {
                    cursors: &mut self.carets, cursor: caret_idx
                };
                self.content.replace_text(&mut self.checkpoints, &mut ce, &movec!());
            }
            if commit {
                self.commit(logs);
            }
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }
    
    fn operate_delete(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.op_no_virtual_spaces();
        if self.carets.any_selected() {
            for caret_idx in 0..self.carets.carets.len() {
                let mut cursor_editor = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
                self.content.replace_text(&mut self.checkpoints, &mut cursor_editor, &movec!())
            }
            self.commit(logs);
        } else {
            let commit = self.carets.carets.iter().any(|c| c.get_position().cursor.col == self.content[c.get_position().cursor.line].len());
            for caret_idx in 0..self.carets.carets.len() {
                self.carets.carets[caret_idx].selection_make_forwards(&self.content);
                let mut ce = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
                self.content.replace_text(&mut self.checkpoints, &mut ce, &movec!());
            }
            if commit {
                self.commit(logs);
            }
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }



    fn operate_tab(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        // todo: unhandled selection
        logs.clear();
        self.op_materialize_virtual_spaces();
        for caret_idx in 0..self.carets.carets.len() {
            let mut cursor_editor = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
            let tab_string = Self::op_get_tab_little_string(&cursor_editor, self.tab_size);
            self.content.replace_text(&mut self.checkpoints, &mut cursor_editor, &movec!(tab_string));
        }
        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_scroll_prev(&mut self, i: u16, logs: &mut Vec<Log>) {
        logs.clear();
        self.scrollbar.prev(i);
    }
    fn operate_scroll_next_line(&mut self, i: usize, logs: &mut Vec<Log>) {
        logs.clear();
        self.scrollbar.next_line(i);
    }
    fn operate_scroll_prev_line(&mut self, i: usize, logs: &mut Vec<Log>) {
        logs.clear();
        self.scrollbar.prev_line(i);
    }
    fn operate_scroll_next(&mut self, i: u16, logs: &mut Vec<Log>) {
        logs.clear();
        self.scrollbar.next(i);
    }
    fn operate_arrow_begin(&mut self, logs: &mut Vec<Log>) {
        logs.clear();
        self.commit(logs);
    }
    fn operate_arrow_end(&mut self, last_content_rect: Rect) {
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }
    
    fn operate_undo(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        self.commit(logs);
        if self.checkpoints.cursor_lened == 0 {
            logs.push(Log {
                message: "[I] No more checkpoints available".to_string(),
                color: C_INFO,
            });
            return;
        }
        logs.clear();

        self.checkpoints.cursor_lened -= 1;
        let checkpoint = &self.checkpoints.others[self.checkpoints.cursor_lened];

        self.carets.carets.resize_with(checkpoint.inner.len(), || Caret::new());
        for (idx, edit) in checkpoint.inner.iter().enumerate().rev() {
            self.carets.carets[idx].set_position(Position::new(
                Cursor::new(
                    edit.edit.start_line,
                    edit.edit.start_col
                ),
                Selection::new(
                    edit.edit.added_data.len().saturating_sub(1) + edit.edit.start_line,
                    edit.edit.added_data.iter().skip(1).last().map(LittleString::len)
                        .unwrap_or(edit.edit.added_data.get(0).map(LittleString::len).unwrap_or(0) + edit.edit.start_col)
                )
            ));
            let mut ce = CursorEditor {
                cursor: idx,
                cursors: &mut self.carets,
            };
            unsafe {
                // These two lines used for debugging:
                // let mut f = File::options().write(true).open("/dev/tty2").unwrap();
                // write!(f, "caret: {:?}\nremoved_data: {:?}\nadded_data: {:?}\n", ce.cursors.carets[ce.cursor], edit.edit.removed_data, edit.edit.added_data).unwrap();
                self.content.replace_text_without_checkpoints(&mut ce, &edit.edit.removed_data)
            }
        }

        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_redo(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        self.commit(logs);
        if self.checkpoints.cursor_lened == self.checkpoints.others.len() {
            logs.push(Log {
                message: "[I] No more checkpoints available".to_string(),
                color: C_INFO,
            });
            return;
        }
        logs.clear();

        let checkpoint = &self.checkpoints.others[self.checkpoints.cursor_lened];
        self.checkpoints.cursor_lened += 1;

        self.carets.carets.resize_with(checkpoint.inner.len(), || Caret::new());
        for (idx, edit) in checkpoint.inner.iter().enumerate().rev() {
            self.carets.carets[idx].set_position(Position::new(
                Cursor::new(
                    edit.edit.start_line,
                    edit.edit.start_col
                ),
                Selection::new(
                    edit.edit.removed_data.len().saturating_sub(1) + edit.edit.start_line,
                    edit.edit.removed_data.iter().skip(1).last().map(LittleString::len)
                        .unwrap_or(edit.edit.removed_data.get(0).map(LittleString::len).unwrap_or(0) + edit.edit.start_col)
                )
            ));
            let mut ce = CursorEditor {
                cursor: idx,
                cursors: &mut self.carets,
            };
            unsafe {
                // These two lines used for debugging:
                // let mut f = File::options().write(true).open("/dev/tty2").unwrap();
                // write!(f, "caret: {:?}\nremoved_data: {:?}\nadded_data: {:?}\n", ce.cursors.carets[ce.cursor], edit.edit.removed_data, edit.edit.added_data).unwrap();
                self.content.replace_text_without_checkpoints(&mut ce, &edit.edit.added_data)
            }
        }

        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_copy(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.op_no_virtual_spaces();
        self.commit(logs);

        self.op_copy(logs);
        
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_cut(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.op_no_virtual_spaces();

        self.commit(logs);
        self.op_copy(logs);
        for caret_idx in 0..self.carets.carets.len() {
            let mut cursor_editor = CursorEditor { cursors: &mut self.carets, cursor: caret_idx };
            self.content.replace_text(&mut self.checkpoints, &mut cursor_editor, &movec!())
        }
        self.commit(logs);

        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_paste(&mut self, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();

        self.commit(logs);
        self.op_materialize_virtual_spaces();
        
        let Some(clipboard) = self.op_get_each_cursor_clipboard(logs)
            else { return; };
        clipboard.into_map_enumerate(|(idx, to_place)| {
            let mut ce = CursorEditor { cursor: idx, cursors: &mut self.carets };
            self.content.replace_text(&mut self.checkpoints, &mut ce, &to_place);
        });

        self.commit(logs);
        
        
        self.buffer_modified();
        self.carets.merge();
        self.carets.ensure_cursors_visible(&mut self.scrollbar, last_content_rect);
    }

    fn operate_save(&mut self, logs: &mut Vec<Log>) {
        self.commit(logs);
        let _ = self.save(None, logs);
    }

    fn operate_single_click(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.commit(logs);

        let x = (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize;
        let y = (row.wrapping_sub(last_content_rect.y)) as usize + self.scrollbar.top_position;
        self.carets.set_cursor(
            x,
            y,
            &self.content
        );
        self.drag_start_pos = (x, y);
    }

    fn operate_add_cursor(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.commit(logs);

        self.carets.add_cursor(
            (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize,
            row.wrapping_sub(last_content_rect.y) as usize + self.scrollbar.top_position,
            &self.content
        );
    }

    fn operate_mouse_select(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.commit(logs);

        let cert_ptr = &mut self.carets.carets[0];
        if cert_ptr.is_selection_none() {
            cert_ptr.set_selection_to_cursor();
        }
        cert_ptr.set_just_cursor(
            (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize,
            row.wrapping_sub(last_content_rect.y) as usize + self.scrollbar.top_position,
            &self.content
        );
    }

    fn operate_multicursor_select(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.commit(logs);

        let x = (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize;
        let y = row.wrapping_sub(last_content_rect.y) as usize + self.scrollbar.top_position;
        let start_line = y.min(self.drag_start_pos.1);
        let end_line = y.max(self.drag_start_pos.1);
        let n_lines = 1 + end_line - start_line;
        self.carets.carets.resize_with(n_lines, Caret::new);
        let mut idx = 0;
        for l in start_line..=end_line {
            self.carets.carets[idx].set_position(Position::new(
                Cursor::new(l, x),
                Selection::new(l, self.drag_start_pos.0)
            ));
            self.carets.carets[idx].merge_sel_pos();
            idx += 1;
        }

    }

    fn operate_double_click(&mut self, col: u16, row: u16, last_content_rect: Rect, logs: &mut Vec<Log>) {
        logs.clear();
        self.commit(logs);

        let x = (col.wrapping_sub(last_content_rect.x) + self.scrollbar.position) as usize;
        let y = (row.wrapping_sub(last_content_rect.y)) as usize + self.scrollbar.top_position;

        let (line, mut start_col) = Cursor::clamp_position(x, y, &self.content);

        let current_char = self.content[line][start_col..].chars().next().unwrap_or('_');
        if !current_char.is_alphanumeric() && current_char != '_' {
            return;
        }

        let mut end_col = start_col;

        while start_col != 0 {
            let char = self.content[line][start_col - 1..].chars().next().unwrap();
            if char.is_alphanumeric() || char == '_' {
                start_col -= 1;
            } else {
                break;
            }
        }

        let line_len = self.content[line].len();
        while end_col < line_len {
            let char = self.content[line][end_col..].chars().next().unwrap();
            if char.is_alphanumeric() || char == '_' {
                end_col += 1;
            } else {
                break;
            }
        }

        // Double click occurs after a single click, so, there should be one cursor
        self.carets.carets[0].set_position(
            Position {
                cursor: Cursor::new(line, end_col),
                selection: Selection::new(line, start_col),
            }
        );
    }
}