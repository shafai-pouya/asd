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

    /// Be careful with using this function. If you use this function when the selection is none,
    /// lc can be incorrect, if you don't change it or don't use sync_cursor_with_selection method.
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
    #[cfg(test)]
    pub unsafe fn set_col(&mut self, col: usize) {
        self.col = col;
    }

    #[inline]
    pub(crate) fn next_or_prev_unchecked(&mut self, n: isize) {
        self.col = (self.col as isize).overflowing_add(n).0 as usize;
    }

    #[inline]
    pub(crate) fn up_or_down_unchecked(&mut self, n: isize) {
        self.line = ((self.line as isize).overflowing_add(n).0) as usize;
    }

    /// This function don't panic when the selection is none, but it may make heavy operations.
    pub(crate) fn validate_wide_chars(&mut self, content: &Content) {
        let Some(ln) = content.get(self.line) else {
            return;
        };
        if let Some(ch) = ln.get(self.col) {
            self.col -= ch.char().char_start_offset()
        }
    }

    /// This function don't panic when the selection is none, but it may make heavy operations.
    pub(crate) fn validate_wide_chars_forwards(&mut self, content: &Content) {
        let Some(ln) = content.get(self.line) else {
            return;
        };
        while let Some(ch) = ln.get(self.col) {
            if ch.char().char_start_offset() == 0 {
                break;
            }
            self.col += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::encoding::Encoding;
    use crate::backend::selection::Selection;

    #[test]
    fn simple_methods() {
        let mut s = Selection::new(1234, 5678);
        assert_eq!(s.get_lc(), (1234, 5678));
        assert_eq!(s.display_line(), 1235);
        assert_eq!(s.display_col(), 5679);
        assert!(!s.is_none());

        s.set_none();
        assert_eq!(s.get_lc(), (1234, 5678));
        assert!(s.is_none());

        unsafe {
            s.set_col(5555);
        }
        assert_eq!(s.get_lc(), (1234, 5555));

        s.next_or_prev_unchecked(10);
        assert_eq!(s.get_lc(), (1234, 5565));
        s.next_or_prev_unchecked(-20);
        assert_eq!(s.get_lc(), (1234, 5545));

        s.up_or_down_unchecked(10);
        assert_eq!(s.get_lc(), (1244, 5545));
        s.up_or_down_unchecked(-20);
        assert_eq!(s.get_lc(), (1224, 5545));
    }

    #[test]
    fn test_validate_wide_chars() {
        let mut selection = Selection::new(0, 0);
        let (_, content) = Encoding::from_str_utf8(include_str!("../assets/test.txt"));

        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (0, 0));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (0, 0));

        (selection.line, selection.col) = (0, 1);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (0, 1));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (0, 1));

        (selection.line, selection.col) = (1, 0);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (1, 0));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (1, 0));

        (selection.line, selection.col) = (1, 1);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (1, 1));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (1, 1));

        // emojis:
        (selection.line, selection.col) = (2, 0);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (2, 0));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (2, 0));

        (selection.line, selection.col) = (2, 1);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (2, 0));
        (selection.line, selection.col) = (2, 1);
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (2, 2));

        (selection.line, selection.col) = (2, 2);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (2, 2));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (2, 2));

        // special:
        (selection.line, selection.col) = (7, 25);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (7, 25));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (7, 25));

        (selection.line, selection.col) = (7, 26);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (7, 25));
        (selection.line, selection.col) = (7, 26);
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (7, 28));

        (selection.line, selection.col) = (7, 27);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (7, 25));
        (selection.line, selection.col) = (7, 26);
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (7, 28));

        (selection.line, selection.col) = (7, 28);
        selection.validate_wide_chars(&content);
        assert_eq!(selection.get_lc(), (7, 28));
        selection.validate_wide_chars_forwards(&content);
        assert_eq!(selection.get_lc(), (7, 28));

        // should not panic when selection is invalid or none:
        (selection.line, selection.col) = (1000, 1000);
        selection.validate_wide_chars(&content);
        selection.validate_wide_chars_forwards(&content);

        (selection.line, selection.col) = (0, 1000);
        selection.validate_wide_chars(&content);
        selection.validate_wide_chars_forwards(&content);

        selection.set_none();
        selection.validate_wide_chars(&content);
        selection.validate_wide_chars_forwards(&content);
    }
}
