use crate::backend::content::Content;
use crate::backend::cursor::Cursor;
use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::{IterMut, MostlyOneVec};
use crate::backend::selection::Selection;
use crate::movec;
use crate::ui::custom_scrollbar::CustomScrollbar;
use ratatui::layout::Rect;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

pub struct CursorEditor<'a> {
    pub cursor: usize,
    pub cursors: &'a mut Carets,
}

impl<'a> CursorEditor<'a> {
    pub(crate) fn move_anything_after_ud_np_included(
        &mut self,
        line: usize,
        col: usize,
        ud: isize,
        np: isize,
    ) {
        for i in &mut *self.cursors {
            if i.position.cursor.get_line() == line && i.position.cursor.get_col() >= col {
                i.position.cursor.next_or_prev_unchecked(np);
                i.position.cursor.up_or_down_unchecked(ud);
            } else if i.position.cursor.get_line() > line {
                i.position.cursor.up_or_down_unchecked(ud);
            }

            if i.position.selection.is_none() {
                continue;
            }

            if i.position.selection.get_line() == line && i.position.selection.get_col() >= col {
                i.position.selection.next_or_prev_unchecked(np);
                i.position.selection.up_or_down_unchecked(ud);
            } else if i.position.selection.get_line() > line {
                i.position.selection.up_or_down_unchecked(ud);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    cursor: Cursor,
    selection: Selection,
}

pub struct Caret {
    position: Position,
    pub started: bool,
    pub added_len: usize,
    pub removed_text: MostlyOneVec<LittleString>,
}

impl Caret {
    pub(crate) fn start_checkpoint(&mut self) {
        if self.started {
            return;
        }
        assert_eq!(self.added_len, 0);
        assert_eq!(self.removed_text.len(), 0);
        self.started = true;
        self.added_len = 0;
        self.removed_text = movec!();
    }
}

impl Caret {
    #[inline]
    pub(crate) fn is_selection_none(&self) -> bool {
        self.position.selection.is_none()
    }

    pub(crate) fn set_selection_to_cursor(&mut self) {
        self.position.selection.none = false;
        self.position.selection.line = self.position.cursor.line;
        self.position.selection.col = self.position.cursor.col;
    }

    pub(crate) fn set_just_cursor_mouse(&mut self, x: usize, y: usize, content: &Content) {
        let (line, col) = Cursor::clamp_position(x, y, content);
        self.position.cursor = Cursor::new(line, col);
        self.position.cursor.validate_wide_chars(content);
    }

    pub(crate) fn selection_make_backwards(&mut self, content: &Content) {
        if self.position.cursor.col == 0 {
            if self.position.cursor.line == 0 {
                return;
            }
            self.position.selection.none = false;
            self.position.selection.line = self.position.cursor.line - 1;
            self.position.selection.col = content[self.position.selection.line].len();
        } else {
            self.position.selection.none = false;
            self.position.selection.line = self.position.cursor.line;
            self.position.selection.col = self.position.cursor.col - 1;
        }
        self.position.selection.validate_wide_chars(content);
    }

    pub(crate) fn selection_make_forwards(&mut self, content: &Content) {
        let caret_ptr = &mut self.position;
        if caret_ptr.cursor.col == content[caret_ptr.cursor.line].len() {
            caret_ptr.selection.line = caret_ptr.cursor.line + 1;
            if caret_ptr.selection.line == content.len() {
                return;
            }
            caret_ptr.selection.col = 0;
            caret_ptr.selection.none = false;
        } else {
            caret_ptr.selection.none = false;
            caret_ptr.selection.line = caret_ptr.cursor.line;
            caret_ptr.selection.col = caret_ptr.cursor.col + 1;
            caret_ptr.selection.validate_wide_chars_forwards(content);
        }
    }
}

impl Position {
    #[inline]
    pub(crate) fn new(cursor: Cursor, selection: Selection) -> Position {
        Position { cursor, selection }
    }
}

impl Caret {
    #[inline]
    pub(crate) unsafe fn set_position_unchecked(&mut self, value: Position) {
        self.position = value;
    }

    #[inline]
    pub(crate) fn get_position(&self) -> &Position {
        &self.position
    }

    #[inline]
    pub(crate) unsafe fn get_position_mut_unchecked(&mut self) -> &mut Position {
        &mut self.position
    }
}

impl Position {
    pub(crate) fn get_min(&self) -> (usize, usize) {
        let selection = (self.selection.line, self.selection.col);
        let cursor = (self.cursor.line, self.cursor.col);
        if self.selection.none || cursor < selection {
            cursor
        } else {
            selection
        }
    }
    #[inline]
    pub(crate) fn set_selection_none(&mut self) {
        self.selection.set_none()
    }
    #[inline]
    pub(crate) fn is_selection_none(&self) -> bool {
        self.selection.is_none()
    }

    #[inline]
    pub(crate) fn set_cursor_line_into_selection_line(&mut self) {
        self.selection.line = self.cursor.line;
    }

    /// Returns 1 more than cursor if there is no selections
    pub(crate) fn get_max(&self, add_one: bool) -> (usize, usize) {
        let selection = (self.selection.line, self.selection.col);
        let cursor = (self.cursor.line, self.cursor.col);
        if self.selection.none {
            return (cursor.0, cursor.1 + add_one as usize);
        }
        if cursor > selection {
            cursor
        } else {
            selection
        }
    }
    pub(crate) fn set_max(&mut self, a: (usize, usize)) {
        let selection = (self.selection.line, self.selection.col);
        let cursor = (self.cursor.line, self.cursor.col);
        if self.selection.none || cursor > selection {
            self.cursor.line = a.0;
            self.cursor.col = a.1;
        } else {
            self.selection.line = a.0;
            self.selection.col = a.1;
        }
    }
    pub(crate) fn set_min(&mut self, a: (usize, usize)) {
        let selection = (self.selection.line, self.selection.col);
        let cursor = (self.cursor.line, self.cursor.col);
        if self.selection.none || cursor < selection {
            self.cursor.line = a.0;
            self.cursor.col = a.1;
        } else {
            self.selection.line = a.0;
            self.selection.col = a.1;
        }
    }

    #[inline]
    pub(crate) fn set_selection_to_cursor(&mut self) {
        self.selection.none = false;
        self.selection.line = self.cursor.line;
        self.selection.col = self.cursor.col;
    }

    #[inline]
    pub(crate) fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    #[inline]
    pub(crate) fn selection(&self) -> &Selection {
        &self.selection
    }
    #[inline]
    pub(crate) unsafe fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    #[inline]
    pub(crate) unsafe fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }
    #[inline]
    pub(crate) fn cursor_up(&mut self, i: usize, content: &Content) {
        self.cursor.up(i, content)
    }
    #[inline]
    pub(crate) fn cursor_down(&mut self, i: usize, content: &Content) {
        self.cursor.down(i, content)
    }
    #[inline]
    pub(crate) fn cursor_prev(&mut self, content: &Content) {
        self.cursor.prev(content)
    }

    #[inline]
    pub(crate) fn cursor_next(&mut self, content: &Content) {
        self.cursor.next(content)
    }

    #[inline]
    pub(crate) fn cursor_prev_word(&mut self, content: &Content) {
        self.cursor.prev_word(content)
    }

    #[inline]
    pub(crate) fn cursor_next_word(&mut self, content: &Content) {
        self.cursor.next_word(content)
    }

    #[inline]
    pub(crate) fn cursor_end(&mut self, content: &Content) {
        self.cursor.end(content)
    }

    #[inline]
    pub(crate) fn cursor_home(&mut self) {
        self.cursor.home()
    }

    #[inline]
    pub(crate) fn cursor_ctrl_end(&mut self, content: &Content) {
        self.cursor.ctrl_end(content)
    }

    #[inline]
    pub(crate) fn cursor_ctrl_home(&mut self) {
        self.cursor.ctrl_home()
    }

    #[inline]
    pub(crate) fn set_cursor_unchecked(&mut self, cursor: Cursor) {
        self.cursor = cursor
    }
}

impl Caret {
    pub(crate) fn merge_sel_pos(&mut self) {
        if !self.position.selection.is_none()
            && self.position.cursor.line == self.position.selection.line
            && self.position.cursor.col == self.position.selection.col
        {
            self.position.set_selection_none();
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_min().cmp(&other.get_min())
    }
}

impl Eq for Position {}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.get_min() == other.get_min()
    }
}

impl PartialOrd for Caret {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Caret {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl Eq for Caret {}
impl PartialEq for Caret {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
    }
}

pub struct Carets {
    pub carets: MostlyOneVec<Caret>,
}

impl Carets {
    pub(crate) fn any_selected(&self) -> bool {
        self.carets
            .iter()
            .any(|caret| !caret.get_position().selection.none)
    }
}

pub struct CertsIter<'a> {
    pub inner: IterMut<'a, Caret>,
}

impl<'a> Iterator for CertsIter<'a> {
    type Item = &'a mut Caret;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a> IntoIterator for &'a mut Carets {
    type Item = &'a mut Caret;
    type IntoIter = CertsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CertsIter {
            inner: self.carets.iter_mut(),
        }
    }
}

impl Display for Carets {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.carets.len().min(3) {
            let pos_ptr = self.carets[i].get_position();
            if pos_ptr.selection.is_none() {
                write!(
                    f,
                    "{}:{} ",
                    pos_ptr.cursor.display_line(),
                    pos_ptr.cursor.display_col()
                )?;
            } else {
                write!(
                    f,
                    "{}:{} TO {}:{} ",
                    pos_ptr.cursor.display_line(),
                    pos_ptr.cursor.display_col(),
                    pos_ptr.selection.display_line(),
                    pos_ptr.selection.display_col(),
                )?;
            }
        }
        if self.carets.len() > 3 {
            write!(f, "…")?;
        }
        Ok(())
    }
}

impl Carets {
    pub(crate) fn new() -> Self {
        Carets {
            carets: movec![Caret::new()],
        }
    }

    /// # Notes:
    /// - merge and sort cursors is preferred
    pub(crate) fn ensure_cursors_visible(
        &self,
        scrollbar: &mut CustomScrollbar,
        last_content_rect: Rect,
    ) {
        for caret in &self.carets {
            let pos = caret.get_position();
            if pos.selection.is_none() {
                continue;
            }
            scrollbar.ensure_cursor_visible(
                pos.selection.col,
                pos.selection.line,
                last_content_rect,
            );
        }
        for caret in &self.carets {
            let pos = caret.get_position();
            scrollbar.ensure_cursor_visible(pos.cursor.col, pos.cursor.line, last_content_rect);
        }
    }

    pub(crate) fn merge(&mut self) {
        self.carets.sort();
        let mut in_list_idx = 1;
        for i in 1..self.carets.len() {
            if self.carets[i - 1].position.get_max(true) > self.carets[i].position.get_min() {
                let max = self.carets[i].position.get_max(true);
                let min = self.carets[i - 1].position.get_min();
                self.carets[in_list_idx - 1].position.set_max(max);
                self.carets[in_list_idx - 1].position.set_min(min);
            } else {
                self.carets[in_list_idx].position = self.carets[i].position;
                in_list_idx += 1;
            }
        }
        self.carets.truncate(in_list_idx);
        // todo: if the prev len not equals to the current len, commit the checkpoint
    }

    pub(crate) fn set_cursor_mouse(&mut self, x: usize, y: usize, content: &Content) {
        self.carets.truncate(1);
        self.carets[0].position.selection.none = true;
        self.carets[0]
            .position
            .cursor
            .set_only_cursor_mouse(x, y, content);
    }

    pub(crate) fn add_cursor(&mut self, mut x: usize, mut y: usize, content: &Content) {
        (x, y) = Cursor::clamp_position(x, y, content);
        let mut cursor = Cursor::new(y, x);
        cursor.validate_wide_chars(content);
        if let Some(idx) = self.carets.iter().position(|p| {
            p.get_position().cursor.line == cursor.line && p.get_position().cursor.col == cursor.col
        }) {
            if self.carets.len() == 1 {
                return;
            }
            self.carets.swap_remove(idx);
        } else {
            let pos = Position {
                cursor,
                selection: Selection::empty(),
            };
            self.carets.push(Caret::new_from(pos));
        }
    }
}

impl Caret {
    pub(crate) fn new() -> Self {
        Self {
            position: Position::new(Cursor::new(0, 0), Selection::empty()),
            started: false,
            added_len: 0,
            removed_text: movec!(),
        }
    }

    #[inline]
    pub(crate) fn new_from(pos: Position) -> Self {
        Self {
            position: pos,
            started: false,
            added_len: 0,
            removed_text: movec!(),
        }
    }
}
