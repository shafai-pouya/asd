use crate::backend::cursor::Cursor;

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
    pub(crate) fn line(&self) -> usize {
        self.line + 1
    }
    
    #[inline]
    pub(crate) fn col(&self) -> usize {
        self.col + 1
    }

    #[inline]
    pub(crate) fn set_to(&mut self, cursor: &Cursor) {
        self.none = false;
        self.line = cursor.line;
        self.col = cursor.col;
    }
    #[inline]
    pub(crate) fn next_or_prev_unchecked(&mut self, n: isize) {
        self.col = (self.col as isize).overflowing_add(n).0 as usize;
    }
    
    #[inline]
    pub(crate) fn up_or_down_unchecked(&mut self, n: isize) {
        self.line = ((self.line as isize).overflowing_add(n).0) as usize;
    }
}