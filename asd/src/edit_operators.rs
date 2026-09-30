use crate::backend::buffer::Buffer;
use crate::backend::caret::CursorEditor;
use crate::backend::little_string::LittleString;
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
            let line_len = self.content[min.0].len();
            if line_len < min.1 {
                let diff = min.1 - line_len;
                self.content.reserve_at_line(min.0, diff);
                self.carets.carets[i].no_virtual_spaces(&self.content);
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
        }
    }

    fn op_no_virtual_spaces(&mut self) {
        for i in 0..self.carets.carets.len() {
            self.carets.carets[i].no_virtual_spaces(&self.content);
        }
    }

    fn op_get_tab_little_string(ce: &CursorEditor, tab_size: usize) -> LittleString {
        let tab_len =
            tab_size - (ce.cursors.carets[ce.cursor].get_position().cursor.col.col % tab_size);
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
