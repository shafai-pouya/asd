use crate::backend::content::Content;
use crate::ui::custom_scrollbar::CustomScrollbar;
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy)]
pub struct CursorCol {
    pub col: usize,
    pub goal: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub line: usize,
    pub col: CursorCol,
}

impl Cursor {
    #[inline]
    pub(crate) fn new(line: usize, col: usize) -> Self {
        Self {
            line,
            col: CursorCol { col, goal: col },
        }
    }

    #[inline]
    pub(crate) fn get_lc(&self) -> (usize, usize) {
        (self.line, self.col.col)
    }

    pub(crate) fn clamp_position(
        terminal_row: u16,
        terminal_col: u16,
        content: &Content,
        scrollbar: &CustomScrollbar,
        last_content_rect: Rect,
        allow_virtual_spaces: bool,
    ) -> (usize, usize) {
        if content.len() == 0 {
            #[cfg(debug_assertions)]
            panic!(
                "You should remove this if statement if you don't see this panic for a long time."
            ); // I think it's impossible for content to have zero lines, because even a file with 0 bytes has one line.
            #[cfg(not(debug_assertions))]
            return (0, 0);
        }

        let x = (terminal_col.wrapping_sub(last_content_rect.x) + scrollbar.left_position) as usize;
        let y = (terminal_row.wrapping_sub(last_content_rect.y)) as usize + scrollbar.top_position;

        let line = y.min(content.len() - 1);
        let pos = if allow_virtual_spaces {
            x
        } else {
            x.min(content[line].len())
        };

        (line, pos)
    }

    #[inline]
    pub(crate) fn display_line(&self) -> usize {
        self.line + 1
    }

    #[inline]
    pub(crate) fn display_col(&self) -> usize {
        self.col.col + 1
    }
    #[inline]
    #[cfg(test)]
    fn set_col(&mut self, col: usize) {
        self.col = CursorCol { col, goal: col };
    }

    pub(crate) fn validate_wide_chars(&mut self, content: &Content) {
        if let Some(ch) = content[self.line].get(self.col.col) {
            self.col.col -= ch.char().char_start_offset()
        }
    }
    pub(crate) fn validate_wide_chars_forwards(&mut self, content: &Content) {
        while let Some(ch) = content[self.line].get(self.col.col) {
            if ch.char().char_start_offset() == 0 {
                break;
            }
            self.col.col += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_only_cursor_mouse(
        &mut self,
        term_col: u16,
        term_row: u16,
        content: &Content,
        scrollbar: &CustomScrollbar,
        last_content_rect: Rect,
    ) {
        let (line, col) = Self::clamp_position(
            term_row,
            term_col,
            content,
            scrollbar,
            last_content_rect,
            false,
        );

        self.line = line;
        self.col = CursorCol { col, goal: col };

        self.validate_wide_chars(content);
    }

    pub(crate) fn next(&mut self, content: &Content) {
        let line_len = content[self.line].len();
        if self.col.col < line_len {
            self.col.col += 1;
            self.validate_wide_chars_forwards(content);
        } else if self.line + 1 < content.len() {
            self.line += 1;
            self.col.col = 0;
        }

        self.col.goal = self.col.col;
    }

    pub(crate) fn next_word(&mut self, content: &Content) {
        let line_len = content[self.line].len();
        if self.col.col < line_len {
            while let Some(c) = content[self.line].get(self.col.col)
                && c.char() == ' '
            {
                self.col.col += 1;
            }
            if let Some(ch) = content[self.line].get(self.col.col) {
                let base_state = ch.char().is_variable_name();
                while self.col.col < line_len {
                    if content[self.line][self.col.col].char().is_variable_name() == base_state {
                        // Safety: self.col is less than line_len
                        self.col.col += 1;
                    } else {
                        break;
                    }
                }
            }
        } else if self.line + 1 < content.len() {
            self.line += 1;
            self.col.col = 0;
        }

        self.validate_wide_chars_forwards(content);

        self.col.goal = self.col.col;
    }

    #[inline]
    pub(crate) fn next_or_prev_unchecked(&mut self, n: isize) {
        self.col.col = (self.col.col as isize).overflowing_add(n).0 as usize;
        self.col.goal = self.col.col;
    }

    #[inline]
    pub(crate) fn up_or_down_unchecked(&mut self, n: isize) {
        self.line = (self.line as isize).overflowing_add(n).0 as usize;
    }

    pub(crate) fn prev(&mut self, content: &Content) {
        if self.col.col > 0 {
            self.col.col = self.col.col.saturating_sub(1);
        } else if self.line > 0 {
            self.line -= 1;
            self.col.col = content[self.line].len();
        }

        self.validate_wide_chars(content);

        self.col.goal = self.col.col;
    }

    pub(crate) fn prev_word(&mut self, content: &Content) {
        if self.col.col > 0 {
            while self.col.col > 0 && content[self.line][self.col.col - 1].char() == ' ' {
                self.col.col -= 1;
            }
            if let Some(ch) = content[self.line].get(self.col.col.overflowing_sub(1).0) {
                let base_state = ch.char().is_variable_name();
                while self.col.col > 0 {
                    if content[self.line][self.col.col - 1]
                        .char()
                        .is_variable_name()
                        == base_state
                    {
                        // Safe if you call op_no_virtual_spaces before
                        self.col.col -= 1;
                    } else {
                        break;
                    }
                }
            }
        } else if self.line > 0 {
            self.line -= 1;
            self.col.col = content[self.line].len();
        }

        self.col.goal = self.col.col;

        self.validate_wide_chars(content);
    }

    pub(crate) fn down(&mut self, i: usize, content: &Content) {
        self.line += i;
        if self.line >= content.len() {
            self.line = content.len() - 1;
        }

        let line_len = content[self.line].len();

        self.col.col = self.col.goal.min(line_len);

        self.validate_wide_chars(content);
    }

    pub(crate) fn up(&mut self, i: usize, content: &Content) {
        self.line = self.line.saturating_sub(i);

        let line_len = content[self.line].len();

        self.col.col = self.col.goal.min(line_len);

        self.validate_wide_chars(content);
    }

    #[inline]
    pub(crate) fn home(&mut self) {
        self.col = CursorCol { col: 0, goal: 0 };
    }

    #[inline]
    pub(crate) fn end(&mut self, content: &Content) {
        self.col = CursorCol {
            col: content[self.line].len(),
            goal: usize::MAX,
        };
    }

    pub(crate) fn ctrl_home(&mut self) {
        self.col = CursorCol { col: 0, goal: 0 };
        self.line = 0;
    }

    pub(crate) fn ctrl_end(&mut self, content: &Content) {
        self.col = CursorCol { col: 0, goal: 0 };
        self.line = content.len() - 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::cursor::Cursor;
    use crate::backend::encoding::Encoding;
    use crate::ui::custom_scrollbar::CustomScrollbar;
    use ratatui::layout::Rect;

    #[test]
    fn simple_methods() {
        let mut c = Cursor::new(10, 20);
        assert_eq!(c.display_col(), 21);
        assert_eq!(c.display_line(), 11);
        assert_eq!(c.get_lc(), (10, 20));

        c.set_col(1234);
        assert_eq!(c.get_lc(), (10, 1234));
        assert_eq!(c.col.goal, 1234);

        c.set_col(20);

        c.up_or_down_unchecked(2);
        assert_eq!(c.get_lc(), (12, 20));

        c.up_or_down_unchecked(-2);
        assert_eq!(c.get_lc(), (10, 20));

        c.next_or_prev_unchecked(-30);
        assert_eq!(c.get_lc(), (10, usize::MAX - 9));

        c.next_or_prev_unchecked(20);
        assert_eq!(c.get_lc(), (10, 10));
    }

    #[test]
    fn movement_next_and_prev() {
        let mut c = Cursor::new(2, 8);
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));
        let last_content_rect = Rect::new(0, 0, 0, 0);
        let scrollbar = CustomScrollbar::new();

        // normal:
        c.next(&content);
        assert_eq!(c.get_lc(), (2, 10));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (3, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (4, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (4, 1));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (5, 0));
        assert_eq!(c.col.goal, c.col.col);

        c.prev(&content);
        assert_eq!(c.get_lc(), (4, 1));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (4, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (3, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (2, 10));
        assert_eq!(c.col.goal, c.col.col);

        // special:
        c.set_only_cursor_mouse(0, 8, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (8, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (8, 2));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (8, 4));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (9, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (9, 2));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (9, 4));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (9, 7));
        assert_eq!(c.col.goal, c.col.col);

        c.prev(&content);
        assert_eq!(c.get_lc(), (9, 4));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (9, 2));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (9, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (8, 4));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (8, 2));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (8, 0));
        assert_eq!(c.col.goal, c.col.col);

        // start of file:
        c.set_only_cursor_mouse(0, 0, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (0, 0));
        assert_eq!(c.col.goal, c.col.col);
        c.prev(&content);
        assert_eq!(c.get_lc(), (0, 0));
        assert_eq!(c.col.goal, c.col.col);

        // end of file:
        c.set_only_cursor_mouse(45, 9, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 45));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, c.col.col);
        c.next(&content);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, c.col.col);
    }

    #[test]
    fn movement_end_and_home() {
        let mut c = Cursor::new(2, 5);
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));

        c.end(&content);
        assert_eq!(c.get_lc(), (2, 10));
        assert_eq!(c.col.goal, usize::MAX);

        c.home();
        assert_eq!(c.get_lc(), (2, 0));
        assert_eq!(c.col.goal, 0);

        c.ctrl_home();
        assert_eq!(c.get_lc(), (0, 0));
        assert_eq!(c.col.goal, 0);

        c.end(&content); // Just to change cursor and goal

        c.ctrl_end(&content);
        assert_eq!(c.get_lc(), (9, 0));
        assert_eq!(c.col.goal, 0);
    }

    #[test]
    fn movement_up_and_down() {
        let mut c = Cursor::new(1, 5);
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));
        let last_content_rect = Rect::new(0, 0, 0, 0);
        let scrollbar = CustomScrollbar::new();

        let c2 = c;
        c.down(1, &content);
        assert_eq!(c.get_lc(), (2, 4));
        assert_eq!(c.col.goal, 5);
        c.up(1, &content);
        assert_eq!(c.get_lc(), c2.get_lc());
        assert_eq!(c.col.goal, c2.col.goal);

        c.set_only_cursor_mouse(2, 0, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (0, 2));
        assert_eq!(c.col.goal, 2);

        c.up(1, &content);
        assert_eq!(c.get_lc(), (0, 2));
        assert_eq!(c.col.goal, 2);

        c.up(10, &content);
        assert_eq!(c.get_lc(), (0, 2));
        assert_eq!(c.col.goal, 2);

        c.set_only_cursor_mouse(2, 9, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 2));
        assert_eq!(c.col.goal, 2);

        c.down(1, &content);
        assert_eq!(c.get_lc(), (9, 2));
        assert_eq!(c.col.goal, 2);

        c.down(10, &content);
        assert_eq!(c.get_lc(), (9, 2));
        assert_eq!(c.col.goal, 2);

        c.set_only_cursor_mouse(0, 0, &content, &scrollbar, last_content_rect);
        c.end(&content);

        assert_eq!(c.get_lc(), (0, 11));
        assert_eq!(c.col.goal, usize::MAX);

        c.down(1, &content);
        assert_eq!(c.get_lc(), (1, 12));
        assert_eq!(c.col.goal, usize::MAX);

        c.down(10, &content);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, usize::MAX);

        c.up(1, &content);
        assert_eq!(c.get_lc(), (8, 4));
        assert_eq!(c.col.goal, usize::MAX);
    }

    #[test]
    fn movement_word_in_one_line() {
        let mut c = Cursor::new(1, 0);
        let (_, content) = Encoding::from_str_utf8("###\r\n   abcd efgh   ijkl lmno\r\n###\r\n");
        let last_content_rect = Rect::new(0, 0, 0, 0);
        let scrollbar = CustomScrollbar::new();

        c.next_word(&content);
        assert_eq!(c.get_lc(), (1, 7));
        assert_eq!(c.col.goal, c.col.col);

        c.next_word(&content);
        assert_eq!(c.get_lc(), (1, 12));
        assert_eq!(c.col.goal, c.col.col);

        c.next_word(&content);
        assert_eq!(c.get_lc(), (1, 19));
        assert_eq!(c.col.goal, c.col.col);

        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 15));
        assert_eq!(c.col.goal, c.col.col);

        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 8));
        assert_eq!(c.col.goal, c.col.col);

        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 3));
        assert_eq!(c.col.goal, c.col.col);

        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 0));
        assert_eq!(c.col.goal, c.col.col);

        // mid-word
        c.set_only_cursor_mouse(5, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 5));
        c.next_word(&content);
        assert_eq!(c.get_lc(), (1, 7));

        c.set_only_cursor_mouse(5, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 5));
        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 3));

        // mid-space
        c.set_only_cursor_mouse(13, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 13));
        c.next_word(&content);
        assert_eq!(c.get_lc(), (1, 19));

        c.set_only_cursor_mouse(13, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 13));
        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 8));

        // almost at the start of line:
        c.set_only_cursor_mouse(1, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 1));
        c.prev_word(&content);
        assert_eq!(c.get_lc(), (1, 0));

        // go next line
        c.set_only_cursor_mouse(10000, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 24));
        c.next_word(&content);
        assert_eq!(c.get_lc(), (2, 0));

        // go prev line
        c.set_only_cursor_mouse(0, 1, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (1, 0));
        c.prev_word(&content);
        assert_eq!(c.get_lc(), (0, 3));

        // can't go back
        c.set_only_cursor_mouse(0, 0, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (0, 0));
        c.prev_word(&content);
        assert_eq!(c.get_lc(), (0, 0));

        // can't go forward
        c.set_only_cursor_mouse(0, 3, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (3, 0));
        c.next_word(&content);
        assert_eq!(c.get_lc(), (3, 0));

        let mut c = Cursor::new(0, 4);
        let (_, content) = Encoding::from_str_utf8("abcd    \r\n\r\n");
        c.next_word(&content);
        assert_eq!(c.get_lc(), (0, 8));
    }

    #[allow(clippy::identity_op)]
    #[allow(clippy::absurd_extreme_comparisons)]
    #[test]
    fn test_clamp_position() {
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));

        let mut scrollbar = CustomScrollbar::new();

        for ((rect_x, rect_y), (left_pos, top_pos)) in [
            ((0, 0), (0, 0)),
            ((6, 5), (0, 1)),
            ((2, 0), (2, 0)),
            ((3, 4), (3, 4)),
        ] {
            let rect = Rect::new(rect_x, rect_y, 0, 0);
            scrollbar.left_position = left_pos;
            scrollbar.top_position = top_pos;

            let top_pos = top_pos as u16;

            if top_pos <= 0 {
                // Avoid subtract overflow
                if left_pos <= 0 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            0 + rect_y - top_pos,
                            0 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (0, 0)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            0 + rect_y - top_pos,
                            0 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (0, 0)
                    );
                }
                if left_pos <= 11 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            0 + rect_y - top_pos,
                            11 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (0, 11)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            0 + rect_y - top_pos,
                            11 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (0, 11)
                    );
                }
                if left_pos <= 12 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            0 + rect_y - top_pos,
                            12 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (0, 11)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            0 + rect_y - top_pos,
                            12 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (0, 12)
                    );
                }
            }

            if top_pos <= 9 {
                // Avoid subtract overflow
                if left_pos <= 0 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            9 + rect_y - top_pos,
                            0 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (9, 0)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            9 + rect_y - top_pos,
                            0 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (9, 0)
                    );
                }
                if left_pos <= 47 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            9 + rect_y - top_pos,
                            47 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (9, 47)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            9 + rect_y - top_pos,
                            47 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (9, 47)
                    );
                }
                if left_pos <= 48 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            9 + rect_y - top_pos,
                            48 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (9, 47)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            9 + rect_y - top_pos,
                            48 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (9, 48)
                    );
                }
            }

            if top_pos <= 10 {
                // Avoid subtract overflow
                if left_pos <= 47 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            10 + rect_y - top_pos,
                            47 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (9, 47)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            10 + rect_y - top_pos,
                            47 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (9, 47)
                    );
                }
                if left_pos <= 48 {
                    // Avoid subtract overflow
                    assert_eq!(
                        Cursor::clamp_position(
                            10 + rect_y - top_pos,
                            48 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (9, 47)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            10 + rect_y - top_pos,
                            48 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (9, 48)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            10 + rect_y - top_pos,
                            46 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            false
                        ),
                        (9, 46)
                    );
                    assert_eq!(
                        Cursor::clamp_position(
                            10 + rect_y - top_pos,
                            46 + rect_x - left_pos,
                            &content,
                            &scrollbar,
                            rect,
                            true
                        ),
                        (9, 46)
                    );
                }
            }
        }
    }

    #[test]
    fn test_set_only_cursor_mouse() {
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));
        let mut c = Cursor::new(0, 0);
        let last_content_rect = Rect::new(0, 0, 0, 0);
        let scrollbar = CustomScrollbar::new();

        c.set_only_cursor_mouse(0, 0, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (0, 0));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(11, 0, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (0, 11));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(12, 0, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (0, 11));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(0, 9, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 0));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(47, 9, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(48, 9, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(47, 10, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(48, 10, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (9, 47));
        assert_eq!(c.col.goal, c.col.col);

        // specials:
        c.set_only_cursor_mouse(25, 7, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (7, 25));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(26, 7, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (7, 25));
        assert_eq!(c.col.goal, 26);

        c.set_only_cursor_mouse(27, 7, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (7, 25));
        assert_eq!(c.col.goal, 27);

        c.set_only_cursor_mouse(28, 7, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (7, 28));
        assert_eq!(c.col.goal, c.col.col);

        // emoji:
        c.set_only_cursor_mouse(0, 2, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (2, 0));
        assert_eq!(c.col.goal, c.col.col);

        c.set_only_cursor_mouse(1, 2, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (2, 0));
        assert_eq!(c.col.goal, 1);

        c.set_only_cursor_mouse(2, 2, &content, &scrollbar, last_content_rect);
        assert_eq!(c.get_lc(), (2, 2));
        assert_eq!(c.col.goal, c.col.col);
    }
}
