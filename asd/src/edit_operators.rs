use crate::backend::buffer::Buffer;
use crate::backend::caret::{CursorEditor, Position};
use crate::backend::cursor::Cursor;
use crate::backend::little_string::LittleString;
use crate::backend::selection::Selection;
use crate::{Clipboard, movec};

/// This trait made to be implemented for only one struct. It made
/// to implement some functions in another file, for readability,
/// and also use self with it
pub trait EditOperators {
    // fn op_place_char(content: &mut FileBuffer, ce: &mut CursorEditor, c: char);
    fn op_materialize_virtual_spaces(&mut self);
    fn op_no_virtual_spaces(&mut self);
    fn op_get_tab_little_string(ce: &CursorEditor, tab_size: usize) -> LittleString;
    fn op_copy(&mut self, clipboard: &mut Clipboard);
    fn op_get_each_cursor_clipboard(&mut self, clipboard: &mut Clipboard) -> Option<Clipboard>;
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
                self.content.reserve_gms_at_line(line, diff);
                unsafe {
                    self.carets.carets[i].set_position_unchecked(Position::new(
                        Cursor::new(line, line_len),
                        Selection::empty(),
                    ));
                }
                unsafe {
                    self.content.replace_text(
                        &mut self.checkpoints,
                        &mut CursorEditor {
                            cursor: i,
                            cursors: &mut self.carets,
                        },
                        movec!(LittleString::from_spaces_repeated(diff)),
                    );
                } // Safety: spaces work on all encodings
                continue;
            }

            let max = self.carets.carets[i].get_position().get_max(false);
            let line = max.0;
            let col = max.1;
            let line_len = self.content[line].len();
            if line_len < col {
                let mut pos = *self.carets.carets[i].get_position();
                pos.set_max((line, line_len));
                unsafe {
                    self.carets.carets[i].set_position_unchecked(pos);
                }
            }
        }
    }

    fn op_no_virtual_spaces(&mut self) {
        for i in 0..self.carets.carets.len() {
            let mut pos = *self.carets.carets[i].get_position();
            if !pos.selection.is_none() {
                let line = pos.selection.get_line();
                let current_col = pos.selection.get_col();
                if self.content[line].len() < current_col {
                    unsafe {
                        pos.selection_mut().set_col(self.content[line].len());
                    }
                }
            }
            let line = pos.cursor.get_line();
            let current_col = pos.cursor.get_col();
            if self.content[line].len() < current_col {
                unsafe {
                    pos.cursor_mut().set_col(self.content[line].len());
                }
            }
            unsafe {
                self.carets.carets[i].set_position_unchecked(pos);
            }
            self.carets.carets[i].merge_sel_pos()
        }
    }

    fn op_get_tab_little_string(ce: &CursorEditor, tab_size: usize) -> LittleString {
        let tab_len =
            tab_size - (ce.cursors.carets[ce.cursor].get_position().cursor.col % tab_size);
        LittleString::from_spaces_repeated(tab_len)
    }

    fn op_copy(&mut self, clipboard: &mut Clipboard) {
        unsafe {
            *clipboard = (
                self.encoding,
                self.content.get_selected_texts_to_copy(&self.carets),
            );
        }
    }

    fn op_get_each_cursor_clipboard(&mut self, clipboard: &mut Clipboard) -> Option<Clipboard> {
        if self.carets.carets.len() == clipboard.1.len() {
            Some(clipboard.clone())
        } else if self.carets.carets.len() == 1 {
            Some((
                clipboard.0,
                movec!(clipboard.1.iter().map(|a| a[0].clone()).collect()),
            ))
        } else {
            None
        }
    }
}
