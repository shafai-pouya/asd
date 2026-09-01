use crate::backend::content::Content;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalGoal {
    Column(usize),
    EndOfLine,
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub goal: HorizontalGoal,
}

impl Cursor {
    #[inline]
    pub(crate) fn new(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            goal: HorizontalGoal::Column(col),
        }
    }

    #[inline]
    pub(crate) fn get_lc(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    pub(crate) fn clamp_position(x: usize, y: usize, content: &Content) -> (usize, usize) {
        if content.len() == 0 {
            return (0, 0);
        }

        let line = y.min(content.len() - 1);
        let pos = x.min(content[line].len());

        (line, pos)
    }

    #[inline]
    pub(crate) fn display_line(&self) -> usize {
        self.line + 1
    }

    #[inline]
    pub(crate) fn display_col(&self) -> usize {
        self.col + 1
    }

    #[inline]
    pub(crate) fn get_line(&self) -> usize {
        self.line
    }

    #[inline]
    pub(crate) fn get_col(&self) -> usize {
        self.col
    }
    #[inline]
    pub(crate) unsafe fn set_col(&mut self, col: usize) {
        self.col = col
    }

    pub(crate) fn validate_wide_chars(&mut self, content: &Content) {
        if let Some(ch) = content[self.line].get(self.col) {
            self.col -= ch.get_idx_diff_to_reach_start()
        }
    }
    pub(crate) fn validate_wide_chars_forwards(&mut self, content: &Content) {
        while let Some(ch) = content[self.line].get(self.col) {
            if ch.get_idx_diff_to_reach_start() == 0 {
                break;
            }
            self.col += 1;
        }
    }

    pub(crate) fn set_only_cursor_mouse(&mut self, x: usize, y: usize, content: &Content) {
        let (line, col) = Self::clamp_position(x, y, content);

        self.line = line;
        self.col = col;
        self.goal = HorizontalGoal::Column(col);

        self.validate_wide_chars(content);
    }

    pub(crate) fn next(&mut self, content: &Content) {
        let line_len = content[self.line].len();
        if self.col < line_len {
            self.col += 1;
            self.validate_wide_chars_forwards(content);
        } else if self.line + 1 < content.len() {
            self.line += 1;
            self.col = 0;
        }

        self.goal = HorizontalGoal::Column(self.col);
    }

    pub(crate) fn next_word(&mut self, content: &Content) {
        let line_len = content[self.line].len();
        if self.col < line_len {
            while content[self.line][self.col] == ' ' {
                self.col += 1;
            }
            if let Some(ch) = content[self.line].get(self.col) {
                let base_state = ch.is_variable_name();
                while self.col < line_len {
                    if content[self.line][self.col].is_variable_name() == base_state {
                        // Safety: self.col is less than line_len
                        self.col += 1;
                    } else {
                        break;
                    }
                }
            }
        } else if self.line + 1 < content.len() {
            self.line += 1;
            self.col = 0;
        }

        self.goal = HorizontalGoal::Column(self.col);
    }

    #[inline]
    pub(crate) fn next_or_prev_unchecked(&mut self, n: isize) {
        self.col = (self.col as isize).overflowing_add(n).0 as usize;
        self.goal = HorizontalGoal::Column(self.col);
    }

    #[inline]
    pub(crate) fn up_or_down_unchecked(&mut self, n: isize) {
        self.line = (self.line as isize).overflowing_add(n).0 as usize;
    }

    pub(crate) fn prev(&mut self, content: &Content) {
        if self.col > 0 {
            self.col = self.col.saturating_sub(1);
        } else if self.line > 0 {
            self.line -= 1;
            self.col = content[self.line].len();
        }

        self.validate_wide_chars(content);

        self.goal = HorizontalGoal::Column(self.col);
    }

    pub(crate) fn prev_word(&mut self, content: &Content) {
        if self.col > 0 {
            while content[self.line]
                .get(self.col)
                .map(|c| *c == ' ')
                .unwrap_or(false)
            {
                self.col -= 1;
            }
            if let Some(ch) = content[self.line].get(self.col - 1) {
                let base_state = ch.is_variable_name();
                while self.col > 0 {
                    if content[self.line][self.col - 1].is_variable_name() == base_state {
                        // Safe if you call op_no_virtual_spaces before
                        self.col -= 1;
                    } else {
                        break;
                    }
                }
            }
        } else if self.line > 0 {
            self.line -= 1;
            self.col = content[self.line].len();
        }

        self.goal = HorizontalGoal::Column(self.col);
    }

    pub(crate) fn down(&mut self, i: usize, content: &Content) {
        self.line += i;
        if self.line >= content.len() {
            self.line = content.len() - 1;
        }

        let line_len = content[self.line].len();

        self.col = match self.goal {
            HorizontalGoal::Column(c) => c.min(line_len),
            HorizontalGoal::EndOfLine => line_len,
        };

        self.validate_wide_chars(content);
    }

    pub(crate) fn up(&mut self, i: usize, content: &Content) {
        self.line = self.line.saturating_sub(i);

        let line_len = content[self.line].len();

        self.col = match self.goal {
            HorizontalGoal::Column(c) => c.min(line_len),
            HorizontalGoal::EndOfLine => line_len,
        };

        self.validate_wide_chars(content);
    }

    #[inline]
    pub(crate) fn home(&mut self) {
        self.col = 0;
        self.goal = HorizontalGoal::Column(0);
    }

    #[inline]
    pub(crate) fn end(&mut self, content: &Content) {
        self.col = content[self.line].len();
        self.goal = HorizontalGoal::EndOfLine;
    }

    pub(crate) fn ctrl_home(&mut self) {
        self.col = 0;
        self.line = 0;
        self.goal = HorizontalGoal::Column(0);
    }

    pub(crate) fn ctrl_end(&mut self, content: &Content) {
        self.col = 0;
        self.line = content.len() - 1;
        self.goal = HorizontalGoal::Column(0);
    }
}
