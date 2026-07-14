use crate::backend::buffer::Buffer;
use crate::backend::caret::{CursorEditor, Position};
use crate::backend::cursor::Cursor;
use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::MostlyOneVec;
use crate::backend::selection::Selection;
use crate::ui::log::Log;
use crate::{movec, App};

/// This trait made to be implemented for only one struct. It made
/// to implement some functions in another file, for readability,
/// and also use self with it
pub trait EditOperators {
    // fn op_place_char(content: &mut FileBuffer, ce: &mut CursorEditor, c: char);
    fn op_materialize_virtual_spaces(&mut self);
    fn op_no_virtual_spaces(&mut self);
    fn op_get_tab_little_string(ce: &CursorEditor, tab_size: usize) -> LittleString;
    fn op_copy(&mut self, logs: &mut Vec<Log>);
    fn op_get_each_cursor_clipboard(&mut self, logs: &mut Vec<Log>) -> Option<MostlyOneVec<MostlyOneVec<LittleString>>>;
}

impl EditOperators for Buffer {
    fn op_materialize_virtual_spaces(&mut self) {
        for i in 0..self.carets.carets.len() {
            let min = self.carets.carets[i].get_position().get_min();
            let line = min.0;
            let current_col = min.1;
            let line_len = self.content[line].len();
            if line_len < current_col {
                let diff = current_col - line_len;
                self.content.reserve_at_line(line, diff);
                self.carets.carets[i].set_position(Position {
                    cursor: Cursor::new(line, line_len),
                    selection: Selection::empty(),
                });
                self.content.replace_text(
                    &mut self.checkpoints,
                    &mut CursorEditor { cursor: i, cursors: &mut self.carets },
                    &movec!(LittleString::from_char_repeated(b' ', diff))
                );
                continue;
            }

            let max = self.carets.carets[i].get_position().get_max(false);
            let line = max.0;
            let col = max.1;
            let line_len = self.content[line].len();
            if line_len < col {
                let mut pos = *self.carets.carets[i].get_position();
                pos.set_max((line, line_len));
                self.carets.carets[i].set_position(pos);
            }
        }
    }

    fn op_no_virtual_spaces(&mut self) {
        for i in 0..self.carets.carets.len() {
            let mut pos = *self.carets.carets[i].get_position();
            if !pos.selection.is_none() {
                let line = pos.selection.line;
                let current_col = pos.selection.col;
                if self.content[line].len() < current_col {
                    pos.selection.col = self.content[line].len();
                }
            }
            let line = pos.cursor.line;
            let current_col = pos.cursor.col;
            if self.content[line].len() < current_col {
                pos.cursor.col = self.content[line].len();
            }
            self.carets.carets[i].set_position(pos);
            self.carets.carets[i].merge_sel_pos()
        }
    }
    
    fn op_get_tab_little_string(ce: &CursorEditor, tab_size: usize) -> LittleString {
        let tab_len = tab_size - (ce.cursors.carets[ce.cursor].get_position().cursor.col % tab_size);
        LittleString::from_char_repeated(b' ', tab_len)
    }

    fn op_copy(&mut self, logs: &mut Vec<Log>) {
        App::op_set_clipboard(self.content.get_copyable_text(&self.carets, self.line_ending).as_bytes(), logs);
    }

    fn op_get_each_cursor_clipboard(&mut self, logs: &mut Vec<Log>) -> Option<MostlyOneVec<MostlyOneVec<LittleString>>> {
        let text = App::op_get_clipboard(logs)?;
        let parts = text.split('\n').map(|s| s.into()).collect::<MostlyOneVec<_>>();
        if parts.len() == self.carets.carets.len() {
            Some(parts.into_map(|s| movec!(s)))
        } else {
            let mut to_return = MostlyOneVec::with_capacity(self.carets.carets.len());
            for _ in 1..self.carets.carets.len() {
                to_return.push(parts.clone());
            }
            to_return.push(parts);
            Some(to_return)
        }
    }
}