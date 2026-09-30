use crate::backend::checkpoint::Checkpoints;
use crate::backend::content::Content;
use crate::backend::cursor::{Cursor, CursorCol};
use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::{IterMut, MostlyOneVec};
use crate::backend::selection::Selection;
use crate::movec;
use crate::ui::custom_scrollbar::CustomScrollbar;
use ratatui::layout::Rect;
use std::fmt::{Display, Formatter};

pub struct CursorEditor<'a> {
    pub cursor: usize,
    pub cursors: &'a mut Carets,
}

impl<'a> CursorEditor<'a> {
    pub(crate) fn move_anything_after_ud_np_included(&mut self, ud: isize, np: isize) {
        self.cursors.carets[self.cursor].unselect_to_the_side(true);
        let Cursor {
            line,
            col: CursorCol { col, .. },
        } = self.cursors.carets[self.cursor].position.cursor;
        for i in &mut *self.cursors {
            if i.position.cursor.line == line && i.position.cursor.col.col >= col {
                i.position.cursor.next_or_prev_unchecked(np);
                i.position.cursor.up_or_down_unchecked(ud);
            } else if i.position.cursor.line > line {
                i.position.cursor.up_or_down_unchecked(ud);
            }

            if i.position.selection.is_none() {
                continue;
            }

            if i.position.selection.line == line && i.position.selection.col >= col {
                i.position.selection.next_or_prev_unchecked(np);
                i.position.selection.up_or_down_unchecked(ud);
            } else if i.position.selection.line > line {
                i.position.selection.up_or_down_unchecked(ud);
            }
        }
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
        if scrollbar.freeze {
            return;
        }

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
            scrollbar.ensure_cursor_visible(pos.cursor.col.col, pos.cursor.line, last_content_rect);
        }
    }

    pub(crate) fn merge(&mut self, checkpoints: &mut Checkpoints, content: &Content) {
        let mut commited = false;
        self.carets
            .sort_by(|a, b| a.position.get_min().cmp(&b.position.get_min()));
        let mut in_list_idx = 1;
        for i in 1..self.carets.len() {
            let prev_max = self.carets[in_list_idx - 1].position.get_max();
            let current_min = self.carets[i].position.get_min();
            if prev_max >= current_min {
                if !commited {
                    commited = true;
                    // This method mutably borrows `self`. It needs access to fields such as
                    // `Caret::added_len` and `Caret::removed_text`, but does not need the
                    // position. Therefore, it is safe to call this function while another
                    // function is performing a change, since the change has not been applied yet.
                    checkpoints.commit(self, content);
                }
                let current_max = self.carets[i].position.get_max();
                let prev_min = self.carets[in_list_idx - 1].position.get_min();

                let selection = if self.carets[in_list_idx - 1].position.selection.is_none()
                    && self.carets[i].position.selection.is_none()
                {
                    Selection::empty()
                } else {
                    Selection::new(prev_min.0, prev_min.1)
                };
                // Changing only the `position` field is safe because the `commit` function
                // resets the other fields of this struct to their initial values.
                self.carets[in_list_idx - 1].position =
                    Position::new(Cursor::new(current_max.0, current_max.1), selection);
            } else {
                // Changing only the `position` field is safe because the `commit` function
                // resets the other fields of this struct to their initial values.
                self.carets[in_list_idx].position = self.carets[i].position;
                in_list_idx += 1;
            }
        }
        self.carets.truncate(in_list_idx);
    }

    /// This function commits by itself
    pub(crate) fn set_cursor_mouse(
        &mut self,
        terminal_row: u16,
        terminal_col: u16,
        content: &Content,
        checkpoints: &mut Checkpoints,
        scrollbar: &CustomScrollbar,
        last_content_rect: Rect,
    ) {
        let (line, col) = Cursor::clamp_position(
            terminal_row,
            terminal_col,
            content,
            scrollbar,
            last_content_rect,
            false,
        );
        self.carets.truncate(1);
        self.plugin_position_set_and_validate(
            0,
            Position::new(Cursor::new(line, col), Selection::empty()),
            content,
            checkpoints,
        );
    }

    /// This function commits by itself
    /// todo: wrong algorithm for duplicate detection: does not detect cursor inside selections.
    pub(crate) fn add_cursor_mouse(
        &mut self,
        terminal_row: u16,
        terminal_col: u16,
        content: &Content,
        checkpoints: &mut Checkpoints,
        scrollbar: &CustomScrollbar,
        last_content_rect: Rect,
    ) {
        let (line, col) = Cursor::clamp_position(
            terminal_row,
            terminal_col,
            content,
            scrollbar,
            last_content_rect,
            false,
        );
        let cursor = Cursor::new(line, col);
        if let Some(idx) = self.carets.iter().position(|p| {
            p.get_position().cursor.line == line && p.get_position().cursor.col.col == col
        }) {
            if self.carets.len() == 1 {
                return;
            }
            checkpoints.commit(self, content);
            self.carets.swap_remove(idx);
        } else {
            self.plugin_position_add_and_validate(
                Caret::new_from(Position::new(cursor, Selection::empty())),
                content,
                checkpoints,
            );
        }
    }
    /// This function commits by itself
    pub(crate) fn plugin_position_set_and_validate(
        &mut self,
        idx: usize,
        value: Position,
        content: &Content,
        checkpoints: &mut Checkpoints,
    ) {
        checkpoints.commit(self, content);
        self.carets[idx].position = value;
        if self.carets[idx].position.selection.is_none() {
            self.carets[idx]
                .position
                .cursor
                .validate_wide_chars(content);
        } else if self.carets[idx].position.cursor.get_lc()
            < self.carets[idx].position.selection.get_lc()
        {
            self.carets[idx]
                .position
                .cursor
                .validate_wide_chars(content);
            self.carets[idx]
                .position
                .selection
                .validate_wide_chars_forwards(content);
        } else {
            self.carets[idx]
                .position
                .selection
                .validate_wide_chars(content);
            self.carets[idx]
                .position
                .cursor
                .validate_wide_chars_forwards(content);
        }
        self.merge(checkpoints, content);
    }

    /// This function commits by itself
    pub(crate) fn plugin_position_add_and_validate(
        &mut self,
        mut value: Caret,
        content: &Content,
        checkpoints: &mut Checkpoints,
    ) {
        checkpoints.commit(self, content);
        if value.position.selection.is_none() {
            value.position.cursor.validate_wide_chars(content);
        } else if value.position.cursor.get_lc() < value.position.selection.get_lc() {
            value.position.cursor.validate_wide_chars(content);
            value
                .position
                .selection
                .validate_wide_chars_forwards(content);
        } else {
            value.position.selection.validate_wide_chars(content);
            value.position.cursor.validate_wide_chars_forwards(content);
        }
        self.carets.push(value);
        self.merge(checkpoints, content);
    }
}

impl<'a> IntoIterator for &'a mut Carets {
    type Item = &'a mut Caret;
    type IntoIter = CaretsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CaretsIter {
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

pub struct Caret {
    position: Position,
    pub added_len: usize,
    pub removed_text: MostlyOneVec<LittleString>,
    #[cfg(debug_assertions)]
    pub started: bool,
}

impl Caret {
    #[cfg(debug_assertions)]
    pub(crate) fn start_checkpoint(&mut self) {
        if !self.started {
            self.started = true;
            assert_eq!(self.added_len, 0);
            assert_eq!(self.removed_text.len(), 0);
        }
    }
    #[cfg(not(debug_assertions))]
    pub(crate) fn start_checkpoint(&mut self) {
        // Nothing
    }
    pub fn sync_selection_with_cursor(&mut self) {
        if self.position.selection.is_none() {
            self.position.selection.line = self.position.cursor.line;
            self.position.selection.col = self.position.cursor.col.col;
        }
    }

    pub fn select(&mut self) {
        if self.position.selection.is_none() {
            self.position.selection.line = self.position.cursor.line;
            self.position.selection.col = self.position.cursor.col.col;
            self.position.selection.none = false;
        }
    }

    pub fn unselect(&mut self) {
        self.position.selection.none = true;
    }

    pub fn unselect_to_the_side(&mut self, unselect_to_left: bool) {
        let cursor = if unselect_to_left {
            self.position.get_min()
        } else {
            self.position.get_max()
        };
        self.position.selection.set_none();
        self.position.cursor.line = cursor.0;
        self.position.cursor.col.col = cursor.1;
        self.position.cursor.col.goal = cursor.1;
    }

    pub(crate) fn select_cursor_mouse(
        &mut self,
        terminal_row: u16,
        terminal_col: u16,
        content: &Content,
        scrollbar: &CustomScrollbar,
        last_content_rect: Rect,
    ) {
        self.select();
        let (line, col) = Cursor::clamp_position(
            terminal_row,
            terminal_col,
            content,
            scrollbar,
            last_content_rect,
            false,
        );
        self.position.cursor = Cursor::new(line, col);
        self.position.cursor.validate_wide_chars(content);
    }

    /// The selection should be unselected. This function does UB on existing selection
    pub(crate) unsafe fn selection_make_backwards(&mut self, content: &Content) {
        debug_assert!(self.position.selection.is_none());
        if self.position.cursor.col.col == 0 {
            if self.position.cursor.line == 0 {
                return;
            }
            self.position.selection.none = false;
            self.position.selection.line = self.position.cursor.line - 1;
            self.position.selection.col = content[self.position.selection.line].len();
        } else {
            self.position.selection.none = false;
            self.position.selection.line = self.position.cursor.line;
            self.position.selection.col = self.position.cursor.col.col - 1;
        }
        self.position.selection.validate_wide_chars(content);
    }

    /// The selection should be unselected. This function does UB on existing selection
    pub(crate) unsafe fn selection_make_forwards(&mut self, content: &Content) {
        let caret_ptr = &mut self.position;
        debug_assert!(caret_ptr.selection.is_none());
        if caret_ptr.cursor.col.col == content[caret_ptr.cursor.line].len() {
            caret_ptr.selection.line = caret_ptr.cursor.line + 1;
            if caret_ptr.selection.line == content.len() {
                return;
            }
            caret_ptr.selection.col = 0;
            caret_ptr.selection.none = false;
        } else {
            caret_ptr.selection.none = false;
            caret_ptr.selection.line = caret_ptr.cursor.line;
            caret_ptr.selection.col = caret_ptr.cursor.col.col + 1;
            caret_ptr.selection.validate_wide_chars_forwards(content);
        }
    }
    pub fn no_virtual_spaces(&mut self, content: &Content) {
        let max_col = content[self.position.cursor.line].len();
        if self.position.cursor.col.col > max_col {
            self.position.cursor.col.col = max_col;
            self.position.cursor.col.goal = max_col;
        }

        if self.position.selection.is_none() {
            return;
        }

        let max_col = content[self.position.selection.line].len();
        if self.position.selection.col > max_col {
            self.position.selection.col = max_col;
        }

        self.merge_sel_pos()
    }
    #[inline]
    pub(crate) fn checkpoint_position_set(&mut self, value: Position) {
        self.position = value;
    }

    #[inline]
    pub(crate) fn get_position(&self) -> &Position {
        &self.position
    }

    pub(crate) fn merge_sel_pos(&mut self) {
        if !self.position.selection.is_none()
            && self.position.cursor.line == self.position.selection.line
            && self.position.cursor.col.col == self.position.selection.col
        {
            self.unselect();
        }
    }
    #[inline]
    pub(crate) fn cursor_up(&mut self, i: usize, content: &Content) {
        self.position.cursor.up(i, content)
    }
    #[inline]
    pub(crate) fn cursor_down(&mut self, i: usize, content: &Content) {
        self.position.cursor.down(i, content)
    }
    #[inline]
    pub(crate) fn cursor_prev(&mut self, content: &Content) {
        self.position.cursor.prev(content)
    }

    #[inline]
    pub(crate) fn cursor_next(&mut self, content: &Content) {
        self.position.cursor.next(content)
    }

    #[inline]
    pub(crate) fn cursor_prev_word(&mut self, content: &Content) {
        self.position.cursor.prev_word(content)
    }

    #[inline]
    pub(crate) fn cursor_next_word(&mut self, content: &Content) {
        self.position.cursor.next_word(content)
    }

    #[inline]
    pub(crate) fn cursor_end(&mut self, content: &Content) {
        self.position.cursor.end(content)
    }

    #[inline]
    pub(crate) fn cursor_home(&mut self) {
        self.position.cursor.home()
    }

    #[inline]
    pub(crate) fn cursor_ctrl_end(&mut self, content: &Content) {
        self.position.cursor.ctrl_end(content)
    }

    #[inline]
    pub(crate) fn cursor_ctrl_home(&mut self) {
        self.position.cursor.ctrl_home()
    }
    #[cfg(not(debug_assertions))]
    pub(crate) fn new() -> Self {
        Self {
            position: Position::new(Cursor::new(0, 0), Selection::empty()),
            added_len: 0,
            removed_text: movec!(),
        }
    }
    #[cfg(debug_assertions)]
    pub(crate) fn new() -> Self {
        Self {
            position: Position::new(Cursor::new(0, 0), Selection::empty()),
            added_len: 0,
            removed_text: movec!(),
            started: false,
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn new_from(pos: Position) -> Self {
        Self {
            position: pos,
            added_len: 0,
            removed_text: movec!(),
        }
    }
    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn new_from(pos: Position) -> Self {
        Self {
            position: pos,
            added_len: 0,
            removed_text: movec!(),
            started: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub cursor: Cursor,
    pub selection: Selection,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.cursor.get_lc() == other.cursor.get_lc()
            && self.selection.none == other.selection.none
            && (self.selection.none || self.selection.get_lc() == other.selection.get_lc())
    }
}

impl Position {
    #[inline]
    pub(crate) fn new(cursor: Cursor, selection: Selection) -> Position {
        Position { cursor, selection }
    }
    pub(crate) fn get_min(&self) -> (usize, usize) {
        let selection = (self.selection.line, self.selection.col);
        let cursor = (self.cursor.line, self.cursor.col.col);
        if self.selection.none || cursor < selection {
            cursor
        } else {
            selection
        }
    }

    pub(crate) fn get_max(&self) -> (usize, usize) {
        let selection = (self.selection.line, self.selection.col);
        let cursor = (self.cursor.line, self.cursor.col.col);
        if self.selection.is_none() || cursor > selection {
            cursor
        } else {
            selection
        }
    }
}

pub struct CaretsIter<'a> {
    pub inner: IterMut<'a, Caret>,
}

impl<'a> Iterator for CaretsIter<'a> {
    type Item = &'a mut Caret;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::caret::{Caret, Carets, CursorEditor, Position};
    use crate::backend::checkpoint::Checkpoints;
    use crate::backend::cursor::{Cursor, CursorCol};
    use crate::backend::encoding::Encoding;
    use crate::backend::mostly_one_vec::MostlyOneVec;
    use crate::backend::selection::Selection;
    use crate::movec;
    use crate::ui::custom_scrollbar::CustomScrollbar;
    use ratatui::layout::Rect;
    use rstest::rstest;
    use std::cmp::Ordering;

    #[test]
    fn get_min_max() {
        let p = Position::new(Cursor::new(1, 2), Selection::empty());
        assert_eq!(p.get_min(), (1, 2));
        assert_eq!(p.get_max(), (1, 2));
        let mut c = Caret::new_from(p);
        c.sync_selection_with_cursor();
        assert_eq!(c.position.selection.line, c.position.cursor.line);
        assert_eq!(c.position.selection.col, c.position.cursor.col.col);
        assert!(c.position.selection.is_none());

        let p = Position::new(Cursor::new(2, 3), Selection::new(2, 5));
        assert_eq!(p.get_min(), (2, 3));
        assert_eq!(p.get_max(), (2, 5));
        let mut c = Caret::new_from(p);
        c.sync_selection_with_cursor();
        assert_eq!(c.position.selection.line, 2);
        assert_eq!(c.position.selection.col, 5);
        assert!(!c.position.selection.is_none());

        let p = Position::new(Cursor::new(2, 5), Selection::new(2, 3));
        assert_eq!(p.get_min(), (2, 3));
        assert_eq!(p.get_max(), (2, 5));
        let mut c = Caret::new_from(p);
        c.sync_selection_with_cursor();
        assert_eq!(c.position.selection.line, 2);
        assert_eq!(c.position.selection.col, 3);
        assert!(!c.position.selection.is_none());
    }

    #[test]
    fn iter() {
        let mut carets = Carets::new();
        carets.carets = movec![
            Caret::new_from(Position::new(Cursor::new(1, 2), Selection::empty())),
            Caret::new_from(Position::new(Cursor::new(2, 3), Selection::new(2, 5))),
        ];

        let mut iter = (&mut carets).into_iter();
        let next = iter.next().unwrap();
        assert_eq!(
            next.position,
            Position::new(Cursor::new(1, 2), Selection::empty())
        );
        next.position = Position::new(Cursor::new(101, 102), Selection::empty());

        let next = iter.next().unwrap();
        assert_eq!(
            next.position,
            Position::new(Cursor::new(2, 3), Selection::new(2, 5))
        );
        next.position = Position::new(Cursor::new(102, 102), Selection::new(102, 105));

        assert!(iter.next().is_none());

        assert_eq!(
            carets.carets[0].position,
            Position::new(Cursor::new(101, 102), Selection::empty())
        );
        assert_eq!(
            carets.carets[1].position,
            Position::new(Cursor::new(102, 102), Selection::new(102, 105))
        );
    }

    #[test]
    fn any_selected() {
        let mut carets = Carets::new();
        carets.carets = movec!(Caret::new(), Caret::new(), Caret::new());

        assert!(!carets.any_selected());

        carets.carets[0].select();
        assert!(carets.any_selected());

        carets.carets[1].select();
        assert!(carets.any_selected());

        carets.carets[0].unselect();
        assert!(carets.any_selected());

        carets.carets[1].unselect();
        assert!(!carets.any_selected());
    }

    #[rstest]
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(5, 5),
            Selection::empty(),
        )),
        Caret::new_from(Position::new(
            Cursor::new(5, 5),
            Selection::empty(),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(5, 5),
            Selection::empty(),
        )),
    ]
    )] // same cursor
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(5, 5),
            Selection::empty(),
        )),
        Caret::new_from(Position::new(
            Cursor::new(5, 6),
            Selection::empty(),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(5, 5),
            Selection::empty(),
        )),
        Caret::new_from(Position::new(
            Cursor::new(5, 6),
            Selection::empty(),
        )),
    ]
    )] // separate cursors
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 5),
            Selection::new(2, 10),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 5),
            Selection::new(2, 10),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 10),
            Selection::new(2, 5),
        )),
    ]
    )] // same selection
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 3),
            Selection::new(2, 8),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 6),
            Selection::new(2, 12),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 12),
            Selection::new(2, 3),
        )),
    ]
    )] // overlapping selections
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 3),
            Selection::new(2, 8),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::new(2, 12),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 12),
            Selection::new(2, 3),
        )),
    ]
    )] // touching selection boundaries
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 3),
            Selection::new(2, 8),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::empty(),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::new(2, 3),
        )),
    ]
    )] // cursor inside selection
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 3),
            Selection::empty(),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 3),
            Selection::new(2, 8),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::new(2, 3),
        )),
    ]
    )] // cursor at selection start
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::empty(),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 3),
            Selection::new(2, 8),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::new(2, 3),
        )),
    ]
    )] // cursor at selection end
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 1),
            Selection::new(2, 4),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 4),
            Selection::new(2, 7),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 7),
            Selection::new(2, 10),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 10),
            Selection::new(2, 1),
        )),
    ]
    )] // chained overlapping selections
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 8),
            Selection::new(2, 12),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 2),
            Selection::new(2, 5),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 4),
            Selection::new(2, 10),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 12),
            Selection::new(2, 2),
        )),
    ]
    )] // unordered overlapping selections
    #[case(
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 10),
            Selection::new(2, 3),
        )),
        Caret::new_from(Position::new(
            Cursor::new(2, 6),
            Selection::new(2, 12),
        )),
    ],
    movec![
        Caret::new_from(Position::new(
            Cursor::new(2, 12),
            Selection::new(2, 3),
        )),
    ]
    )] // reversed selection
    fn test_merge_cursors(
        #[case] carets_vec: MostlyOneVec<Caret>,
        #[case] expected: MostlyOneVec<Caret>,
    ) {
        let should_commit = carets_vec.len() != expected.len();
        let mut carets = Carets::new();
        carets.carets = carets_vec;

        for c in &mut carets.carets {
            c.added_len = 1; // To test if the code will commit or not
        }

        let (_, content) = Encoding::from_str_utf8(
            "abcdefghijklmnopqrst\nabcdefghijklmnopqrst\nabcdefghijklmnopqrst\nabcdefghijklmnopqrst\nabcdefghijklmnopqrst\nabcdefghijklmnopqrst",
        );

        let mut checkpoints = Checkpoints::new();
        carets.merge(&mut checkpoints, &content);

        for (a, b) in carets.carets.iter().zip(expected) {
            assert_eq!(a.position, b.position);
            assert!(b.removed_text.is_empty());
            assert_eq!(b.added_len, 0);
            if should_commit {
                assert_eq!(a.added_len, 0);
                assert!(a.removed_text.is_empty());
            }
        }

        if should_commit {
            assert!(!checkpoints.others.is_empty());
        } else {
            assert!(checkpoints.others.is_empty());
        }
    }

    #[test]
    fn test_merge_sel_pos() {
        let mut caret = Caret::new_from(Position::new(Cursor::new(2, 2), Selection::empty()));
        caret.merge_sel_pos();
        assert_eq!(
            caret.position,
            Position::new(Cursor::new(2, 2), Selection::empty())
        );

        let mut caret = Caret::new_from(Position::new(Cursor::new(2, 2), Selection::new(2, 2)));
        caret.merge_sel_pos();
        assert_eq!(
            caret.position,
            Position::new(Cursor::new(2, 2), Selection::empty())
        );

        let mut caret = Caret::new_from(Position::new(Cursor::new(2, 2), Selection::new(2, 5)));
        caret.merge_sel_pos();
        assert_eq!(
            caret.position,
            Position::new(Cursor::new(2, 2), Selection::new(2, 5))
        );
    }

    #[rstest]
    #[case((0, 0), None, (0, 0), None)]
    #[case((0, 8), None, (0, 5), None)]
    #[case((1, 20), None, (1, 11), None)]
    #[case((2, 4), None, (2, 0), None)]
    #[case((3, 3), None, (3, 3), None)]
    #[case((0, 9), Some((0, 2)), (0, 5), Some((0, 2)))]
    #[case((1, 4), Some((0, 8)), (1, 4), Some((0, 5)))]
    #[case((0, 8), Some((1, 20)), (0, 5), Some((1, 11)))]
    #[case((0, 8), Some((0, 9)), (0, 5), None)]
    #[case((1, 4), Some((2, 5)), (1, 4), Some((2, 0)))]
    #[case((3, 3), Some((3, 10)), (3, 3), None)]
    fn no_virtual_spaces(
        #[case] c: (usize, usize),
        #[case] s: Option<(usize, usize)>,
        #[case] c2: (usize, usize),
        #[case] s2: Option<(usize, usize)>,
    ) {
        let mut caret = Caret::new_from(Position::new(
            Cursor::new(c.0, c.1),
            if let Some((l, c)) = s {
                Selection::new(l, c)
            } else {
                Selection::empty()
            },
        ));

        let expected_pos = Position::new(
            Cursor::new(c2.0, c2.1),
            if let Some((l, c)) = s2 {
                Selection::new(l, c)
            } else {
                Selection::empty()
            },
        );

        let (_, content) = Encoding::from_str_utf8("hello\nhello world\n\nasd");

        caret.no_virtual_spaces(&content);

        assert_eq!(caret.position, expected_pos);
    }

    #[test]
    fn test_set_position_by_plugin() {
        let lc_list = [
            ((0, 0), (0, 0), (0, 0)),
            ((0, 4), (0, 4), (0, 4)),
            ((0, 11), (0, 11), (0, 11)),
            ((0, 12), (0, 12), (0, 12)),
            ((1, 0), (1, 0), (1, 0)),
            ((1, 4), (1, 4), (1, 4)),
            ((1, 11), (1, 11), (1, 11)),
            ((1, 12), (1, 12), (1, 12)),
            ((2, 0), (2, 0), (2, 0)),
            ((2, 1), (2, 0), (2, 2)),
            ((2, 2), (2, 2), (2, 2)),
            ((2, 3), (2, 2), (2, 4)),
            ((2, 20), (2, 20), (2, 20)),
            ((3, 0), (3, 0), (3, 0)),
            ((3, 1), (3, 1), (3, 1)),
            ((8, 0), (8, 0), (8, 0)),
            ((8, 1), (8, 0), (8, 2)),
            ((8, 2), (8, 2), (8, 2)),
            ((8, 3), (8, 2), (8, 4)),
            ((8, 4), (8, 4), (8, 4)),
            ((9, 0), (9, 0), (9, 0)),
            ((9, 4), (9, 4), (9, 4)),
            ((9, 5), (9, 4), (9, 7)),
            ((9, 6), (9, 4), (9, 7)),
            ((9, 7), (9, 7), (9, 7)),
            ((9, 8), (9, 7), (9, 10)),
            ((9, 9), (9, 7), (9, 10)),
            ((9, 10), (9, 10), (9, 10)),
        ];

        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));
        let mut checkpoints = Checkpoints::new();

        for (lc_cursor, left_lc_cursor, right_lc_cursor) in lc_list {
            for (lc_selection, left_lc_selection, right_lc_selection) in lc_list {
                let pos = if lc_cursor.cmp(&lc_selection) == Ordering::Equal {
                    Position::new(Cursor::new(lc_cursor.0, lc_cursor.1), Selection::empty())
                } else {
                    Position::new(
                        Cursor::new(lc_cursor.0, lc_cursor.1),
                        Selection::new(lc_selection.0, lc_selection.1),
                    )
                };

                let c = Caret::new_from(pos);
                let mut carets = Carets::new();
                carets.carets = movec![c];

                carets.plugin_position_set_and_validate(0, pos, &content, &mut checkpoints);

                let pos = carets.carets[0].position;

                match lc_cursor.cmp(&lc_selection) {
                    Ordering::Less => {
                        assert_eq!(left_lc_cursor, pos.cursor.get_lc());
                        assert_ne!(left_lc_cursor, right_lc_selection);
                        assert!(!pos.selection.is_none());
                        assert_eq!(pos.selection.get_lc(), right_lc_selection);
                    }
                    Ordering::Equal => {
                        assert_eq!(left_lc_cursor, pos.cursor.get_lc());
                        assert!(pos.selection.is_none());
                    }
                    Ordering::Greater => {
                        assert_eq!(right_lc_cursor, pos.cursor.get_lc());
                        assert_ne!(right_lc_cursor, left_lc_selection);
                        assert!(!pos.selection.is_none());
                        assert_eq!(pos.selection.get_lc(), left_lc_selection);
                    }
                }
            }
        }
    }

    #[test]
    fn test_set_cursor_mouse() {
        let last_content_rect = Rect::new(0, 0, 0, 0);
        let scrollbar = CustomScrollbar::new();
        let mut checkpoints = Checkpoints::new();
        let mut carets = Carets::new();
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));

        for ((row, col), (expected_line, expected_col), virtual_spaces) in [
            ((0, 1), (0, 1), false),
            ((0, 11), (0, 11), false),
            ((0, 12), (0, 11), true),
            ((2, 0), (2, 0), false),
            ((2, 1), (2, 0), false),
            ((2, 2), (2, 2), false),
            ((2, 3), (2, 2), false),
            ((7, 25), (7, 25), false),
            ((7, 26), (7, 25), false),
            ((7, 27), (7, 25), false),
            ((7, 28), (7, 28), false),
            ((10, 1), (9, 0), false),
            ((10, 2), (9, 2), false),
            ((10, 48), (9, 47), true),
            ((11, 2), (9, 2), false),
        ] {
            carets.carets = movec![
                Caret::new_from(Position::new(
                    Cursor::new(1234, 5678),
                    Selection::new(1234, 5678)
                )),
                Caret::new_from(Position::new(
                    Cursor::new(1222, 3444),
                    Selection::new(5666, 7888)
                )),
            ];
            assert_ne!(carets.carets.len(), 1, "Test result maybe incorrect");

            carets.set_cursor_mouse(
                row,
                col,
                &content,
                &mut checkpoints,
                &scrollbar,
                last_content_rect,
            );
            assert_eq!(
                carets.carets[0].position,
                Position::new(
                    Cursor::new(expected_line as usize, expected_col as usize),
                    Selection::empty()
                )
            );
            carets.add_cursor_mouse(
                row,
                col,
                &content,
                &mut checkpoints,
                &scrollbar,
                last_content_rect,
            );
            assert_eq!(carets.carets.len(), 1);
            carets.set_cursor_mouse(
                0,
                0,
                &content,
                &mut checkpoints,
                &scrollbar,
                last_content_rect,
            );
            carets.add_cursor_mouse(
                row,
                col,
                &content,
                &mut checkpoints,
                &scrollbar,
                last_content_rect,
            );
            assert_eq!(
                carets.carets[1].position,
                Position::new(
                    Cursor::new(expected_line as usize, expected_col as usize),
                    Selection::empty()
                )
            );
            carets.add_cursor_mouse(
                row,
                col,
                &content,
                &mut checkpoints,
                &scrollbar,
                last_content_rect,
            );
            assert_eq!(
                carets.carets.len(),
                if col != expected_col && !virtual_spaces {
                    2
                } else {
                    1
                }
            );

            carets.add_cursor_mouse(
                row,
                col,
                &content,
                &mut checkpoints,
                &scrollbar,
                last_content_rect,
            );
            let (f_line, f_col) =
                Cursor::clamp_position(row, col, &content, &scrollbar, last_content_rect, true);
            carets.plugin_position_add_and_validate(
                Caret::new_from(Position::new(
                    Cursor::new(f_line, f_col),
                    Selection::empty(),
                )),
                &content,
                &mut checkpoints,
            );
            assert_eq!(carets.carets.len(), if virtual_spaces { 3 } else { 2 });
        }
    }

    #[test]
    fn selection() {
        let last_content_rect = Rect::new(0, 0, 0, 0);
        let scrollbar = CustomScrollbar::new();
        let mut checkpoints = Checkpoints::new();
        let mut carets = Carets::new();
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));

        let test_cases = [
            ((0, 1), (0, 1)),
            ((0, 11), (0, 11)),
            ((0, 12), (0, 11)),
            ((2, 0), (2, 0)),
            ((2, 1), (2, 0)),
            ((2, 2), (2, 2)),
            ((2, 3), (2, 2)),
            ((7, 25), (7, 25)),
            ((7, 26), (7, 25)),
            ((7, 27), (7, 25)),
            ((7, 28), (7, 28)),
            ((10, 1), (9, 0)),
            ((10, 2), (9, 2)),
            ((11, 2), (9, 2)),
            ((10, 48), (9, 47)),
        ];

        for ((row1, col1), (expected_row1, expected_col1)) in test_cases {
            for ((row2, col2), (expected_row2, expected_col2)) in test_cases {
                for ((row3, col3), (expected_row3, expected_col3)) in test_cases {
                    carets.set_cursor_mouse(
                        row1,
                        col1,
                        &content,
                        &mut checkpoints,
                        &scrollbar,
                        last_content_rect,
                    );
                    assert_eq!(
                        carets.carets[0].position,
                        Position::new(
                            Cursor::new(expected_row1, expected_col1),
                            Selection::empty()
                        )
                    );

                    carets.carets[0].select_cursor_mouse(
                        row2,
                        col2,
                        &content,
                        &scrollbar,
                        last_content_rect,
                    );
                    assert_eq!(
                        carets.carets[0].position,
                        Position::new(
                            Cursor::new(expected_row2, expected_col2),
                            Selection::new(expected_row1, expected_col1)
                        )
                    );

                    carets.carets[0].select_cursor_mouse(
                        row3,
                        col3,
                        &content,
                        &scrollbar,
                        last_content_rect,
                    );
                    assert_eq!(
                        carets.carets[0].position,
                        Position::new(
                            Cursor::new(expected_row3, expected_col3),
                            Selection::new(expected_row1, expected_col1)
                        )
                    );
                }
            }
        }

        for (idx1, ((row1, col1), (ex_row1, ex_col1))) in test_cases.into_iter().enumerate() {
            for &((row2, col2), (ex_row2, ex_col2)) in test_cases[idx1 + 1..].iter() {
                // unselect to the left
                carets.set_cursor_mouse(
                    row1,
                    col1,
                    &content,
                    &mut checkpoints,
                    &scrollbar,
                    last_content_rect,
                );
                assert_eq!(
                    carets.carets[0].position,
                    Position::new(Cursor::new(ex_row1, ex_col1), Selection::empty())
                );

                carets.carets[0].select_cursor_mouse(
                    row2,
                    col2,
                    &content,
                    &scrollbar,
                    last_content_rect,
                );
                assert_eq!(
                    carets.carets[0].position,
                    Position::new(
                        Cursor::new(ex_row2, ex_col2),
                        Selection::new(ex_row1, ex_col1)
                    )
                );

                carets.carets[0].unselect_to_the_side(true);
                assert_eq!(
                    carets.carets[0].position,
                    Position::new(Cursor::new(ex_row1, ex_col1), Selection::empty())
                );

                // unselect to the right
                carets.set_cursor_mouse(
                    row1,
                    col1,
                    &content,
                    &mut checkpoints,
                    &scrollbar,
                    last_content_rect,
                );
                assert_eq!(
                    carets.carets[0].position,
                    Position::new(Cursor::new(ex_row1, ex_col1), Selection::empty())
                );

                carets.carets[0].select_cursor_mouse(
                    row2,
                    col2,
                    &content,
                    &scrollbar,
                    last_content_rect,
                );
                assert_eq!(
                    carets.carets[0].position,
                    Position::new(
                        Cursor::new(ex_row2, ex_col2),
                        Selection::new(ex_row1, ex_col1)
                    )
                );

                carets.carets[0].unselect_to_the_side(false);
                assert_eq!(
                    carets.carets[0].position,
                    Position::new(Cursor::new(ex_row2, ex_col2), Selection::empty())
                );
            }
        }
    }

    /// Defined to prevent receiving type complexity warnings from clippy
    type PositionVecType = Vec<((usize, usize), Option<(usize, usize)>)>;

    #[rstest]
    #[case(
        vec![((0, 2), None), ((0, 5), None), ((0, 10), None)],
        vec![((0, 2), None), ((1, 8), None), ((1, 13), None)],
        1, 3, 1
    )]
    #[case(
        vec![((0, 1), Some((0, 4))), ((0, 5), None)],
        vec![((2, 2), None), ((2, 6), None)],
        2, 1, 0
    )]
    #[case(
        vec![((0, 3), None), ((1, 2), None), ((2, 0), Some((2, 5)))],
        vec![((0, 3), None), ((5, 4), None), ((6, 0), Some((6, 5)))],
        4, 2, 1
    )]
    #[case(
        vec![((0, 0), Some((0, 10))), ((0, 5), None)],
        vec![((0, 0), Some((1, 11))), ((1, 6), None)],
        1, 1, 1
    )]
    #[case(
        vec![((0, 5), Some((0, 8)))],
        vec![((2, 8), None)],
        2, 3, 0
    )]
    #[case(
        vec![((0, 1), None), ((0, 4), Some((1, 2)))],
        vec![((0, 1), None), ((0, 4), Some((1, 2)))],
        0, 0, 0
    )]
    #[case(
        vec![((0, 5), None), ((0, 6), None)],
        vec![((3, 7), None), ((3, 8), None)],
        3, 2, 0
    )]
    #[case(
        vec![((0, 0), None), ((2, 1), Some((5, 9)))],
        vec![((2, 6), None), ((4, 1), Some((7, 9)))],
        2, 6, 0
    )]
    #[case(
        vec![((0,5), None), ((0,10), None)],
        vec![((1,7), None), ((1,12), None)],
        1, 2, 0
    )]
    #[case(
        vec![((0,2), None), ((0,10), None)],
        vec![((0,2), None), ((0,13), None)],
        0, 3, 1
    )]
    #[case(
        vec![((0,5), None), ((2,1), None)],
        vec![((1,9), None), ((3,1), None)],
        1, 4, 0
    )]
    #[case(
        vec![((0,3), Some((5, 1))), ((5,2), None)],
        vec![((0,3), Some((5, 1))), ((7,3), None)],
        2, 1, 1
    )]
    #[case(
        vec![((3,8), None), ((3,15), None), ((5,0), None)],
        vec![((2,5), None), ((2, 12), None), ((4,0), None)],
        -1, -3, 0
    )]
    #[case(
        vec![((0,5), Some((0,9))), ((0,12), Some((0,20)))],
        vec![((1,7), None), ((1,14), Some((1,22)))],
        1, 2, 0
    )]
    #[case(
        vec![
            ((1,3), None),
            ((2,4), Some((2,3))),
            ((2,9), None),
            ((4,1), Some((4,5))),
        ],
        vec![
            ((1,3), None),
            ((4,4), None),
            ((4,10), None),
            ((6,1), Some((6,5))),
        ],
        2, 1, 1
    )]
    fn test_ud_np(
        #[case] vec: PositionVecType,
        #[case] expected: PositionVecType,
        #[case] ud: isize,
        #[case] np: isize,
        #[case] idx: usize,
    ) {
        let mut carets = Carets::new();
        carets.carets = vec
            .into_iter()
            .map(|((row1, col1), sel)| {
                Caret::new_from(Position::new(
                    Cursor::new(row1, col1),
                    sel.map(|(row2, col2)| Selection::new(row2, col2))
                        .unwrap_or(Selection::empty()),
                ))
            })
            .collect();

        let mut a = CursorEditor {
            cursors: &mut carets,
            cursor: idx,
        };
        a.move_anything_after_ud_np_included(ud, np);

        assert_eq!(
            carets
                .carets
                .into_iter()
                .map(
                    |Caret {
                         position:
                             Position {
                                 cursor:
                                     Cursor {
                                         line,
                                         col: CursorCol { col, .. },
                                     },
                                 selection,
                             },
                         ..
                     }| (
                        (line, col),
                        if selection.is_none() {
                            None
                        } else {
                            Some((selection.line, selection.col))
                        }
                    )
                )
                .collect::<Vec<_>>(),
            expected
        );
    }
}
