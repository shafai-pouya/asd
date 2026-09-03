use crate::backend::content::Content;

#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub line: usize,
    pub col: usize,
    pub none: bool,
}

impl Selection {
    #[inline]
    pub(crate) fn new(line: usize, col: usize) -> Selection {
        Selection {
            line,
            col,
            none: false,
        }
    }

    #[inline]
    pub(crate) fn get_lc(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

impl Selection {
    #[inline]
    pub(crate) fn empty() -> Self {
        Self {
            line: 0,
            col: 0,
            none: true,
        }
    }

    #[inline]
    pub(crate) fn is_none(&self) -> bool {
        self.none
    }
    #[inline]
    pub(crate) fn set_none(&mut self) {
        self.none = true;
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

    #[inline]
    pub(crate) fn next_or_prev_unchecked(&mut self, n: isize) {
        self.col = (self.col as isize).overflowing_add(n).0 as usize;
    }

    #[inline]
    pub(crate) fn up_or_down_unchecked(&mut self, n: isize) {
        self.line = ((self.line as isize).overflowing_add(n).0) as usize;
    }

    pub(crate) fn validate_wide_chars(&mut self, content: &Content) {
        if let Some(ch) = content[self.line].get(self.col) {
            self.col -= ch.char().get_idx_diff_to_reach_start()
        }
    }

    pub(crate) fn validate_wide_chars_forwards(&mut self, content: &Content) {
        while let Some(ch) = content[self.line].get(self.col) {
            if ch.char().get_idx_diff_to_reach_start() == 0 {
                break;
            }
            self.col += 1;
        }
    }
}
