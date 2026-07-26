use crate::assets::constants::DURATION_SMALL_TIMER;
use crate::backend::caret::{Carets, CursorEditor};
use crate::backend::checkpoint::checkpoints::Checkpoints;
use crate::backend::cursor::Cursor;
use crate::backend::display_string::{DisplaySlice, DisplayString};
use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::MostlyOneVec;
use std::ops::{Index, Range};
use std::time::Instant;

pub(crate) fn whitespaces_in_the_start_of_the_line(s: &DisplayString) -> &DisplaySlice {
    let mut idx = 0;
    for c in s.iter() {
        if !c.is_whitespace() {
            break;
        }
        idx += 1;
    }

    &s[..idx]
}
pub struct Content {
    lines: Vec<DisplayString>,
}

impl Index<usize> for Content {
    type Output = DisplayString;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}


impl Content {
    #[inline]
    pub(crate) fn reserve_gms_at_line(&mut self, line: usize, cap: usize) {
        self.lines[line].reserve(cap);
    }

    #[inline]
    pub(crate) fn from_lines(lines: Vec<DisplayString>) -> Self {
        Content { lines }
    }

    #[inline]
    pub(crate) fn get(&self, i: Range<usize>) -> Option<&[DisplayString]> {
        self.lines.get(i)
    }
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }
    
    pub(crate) fn get_max_line_length(&self) -> usize {
        self.lines.iter().map(|l| l.len()).max().unwrap() // Ok
    }


    /// Safety: Make sure the encoding is correct
    pub(crate) unsafe fn replace_text(&mut self, checkpoints: &mut Checkpoints, carets: &mut CursorEditor, new_text: MostlyOneVec<LittleString>) {
        let caret_ptr = &mut carets.cursors.carets[carets.cursor];
        let new_text_lines_n = new_text.len();
        if caret_ptr.is_selection_none() {
            unsafe {
                let pos_ptr = caret_ptr.get_position_mut_unchecked();
                pos_ptr.set_cursor_line_into_selection_line();
            }
        }
        let pos_ptr = caret_ptr.get_position();
        let min = pos_ptr.get_min();
        let max = pos_ptr.get_max(false);

        // Do checkpoints:
        caret_ptr.start_checkpoint();
        let forward = caret_ptr.get_position().selection().get_lc() > caret_ptr.get_position().cursor().get_lc();
        let mut skip = if !forward { caret_ptr.added_len } else { 0 };
        if caret_ptr.removed_text.is_empty() {
            caret_ptr.removed_text.push(LittleString::empty());
        }

        let range: Box<dyn Iterator<Item=usize>> = if forward { Box::new(min.0..=max.0) } else { Box::new((min.0..=max.0).rev()) };
        for line_idx in range {
            let part = if line_idx == min.0 && line_idx == max.0 {
                &self[line_idx][min.1..max.1]
            } else if line_idx == min.0 {
                &self[line_idx][min.1..]
            } else if line_idx == max.0 {
                &self[line_idx][..max.1]
            } else {
                &self[line_idx]
            };

            if forward {
                for &ch in part {
                    if skip > 0 { skip -= 1; } else {
                        unsafe {
                            caret_ptr.removed_text.last_mut().unwrap().push(ch); // It's Ok
                        } // Safety: the caller
                    }
                }
            } else {
                for ch in part.iter().rev() {
                    if skip > 0 { skip -= 1; } else {
                        unsafe {
                            caret_ptr.removed_text[0].insert(0, *ch);
                        } // Safety: the caller
                    }
                }
            }

            if forward && line_idx != max.0 {
                if skip > 0 { skip -= 1; } else {
                    caret_ptr.removed_text.push(LittleString::empty());
                }
            } else if !forward && line_idx != min.0 {
                if skip > 0 { skip -= 1; } else {
                    caret_ptr.removed_text.insert(0, LittleString::empty());
                }
            }
        }
        if !forward {
            caret_ptr.added_len = skip;
        }
        caret_ptr.added_len += new_text_lines_n.saturating_sub(1) + new_text.iter().map(|l| l.len()).sum::<usize>();

        unsafe { self.replace_text_without_checkpoints(carets, new_text); } // Safety: I did operate checkpoints by myself

        checkpoints.little_timer_deadline = Some(Instant::now() + DURATION_SMALL_TIMER);
    }

    /// Safety: Make sure the encoding is correct, and make sure you don't want to apply it to the
    /// checkpoints, and you have commited checkpoints before
    pub(crate) unsafe fn replace_text_without_checkpoints(&mut self, carets: &mut CursorEditor, new_text: MostlyOneVec<LittleString>) {
        let caret_ptr = &mut carets.cursors.carets[carets.cursor];
        if caret_ptr.is_selection_none() {
            unsafe {
                let pos_ptr = caret_ptr.get_position_mut_unchecked();
                pos_ptr.set_cursor_line_into_selection_line()
            }
        }
        let new_text_lines_n = new_text.len();
        let pos_ptr = caret_ptr.get_position();
        let min = pos_ptr.get_min();
        let max = pos_ptr.get_max(false);
        let selected_text_lines_n = max.0 - min.0 + 1;
        let last_new_line_len = if new_text_lines_n == 0 { 0 } else {
            new_text[new_text_lines_n - 1].len()
        };

        match (new_text_lines_n, selected_text_lines_n) {
            (0, 0) |
            (0, 1) |
            (1, 0) |
            (1, 1) => {
                self.lines[pos_ptr.cursor().line].replace_range(
                    min.1..max.1,
                    new_text.into_iter().next().unwrap_or(LittleString::empty())
                );
            }
            (0, _) |
            (1, _) => {
                let last_line = self.lines
                    .drain(min.0 + 1..max.0 + 1).last().unwrap(); // We checked the len in the match case

                self.lines[min.0].truncate(min.1);
                unsafe {
                    self.lines[min.0].push_slice(new_text.get(0).map(AsRef::as_ref).unwrap_or(Default::default()));
                    self.lines[min.0].push_slice(&last_line[max.1..]);
                } // Safety: The caller
            }
            (_, 0) |
            (_, 1) => {
                let mut new_text = new_text.into_iter();
                let first_new_line = new_text.next().unwrap(); // We checked the length in the match case
                self.lines.splice(
                    min.0 + 1..min.0 + 1,
                    new_text.map(LittleString::into_dstring)
                );
                let dst = min.0 + new_text_lines_n - 1;
                let (left, right) = self.lines.split_at_mut(dst);
                unsafe {
                    right[0].push_slice(&left[min.0][max.1..]);
                    self.lines[min.0].truncate(min.1);
                    self.lines[min.0].push_slice(first_new_line.as_ref());
                } // Safety: The caller
            }
            (_, _) => {
                let mut new_text = new_text.into_iter();
                let first_new_line = new_text.next().unwrap();
                let last_old_line = self.lines.splice(
                    min.0 + 1..max.0 + 1,
                    new_text.map(LittleString::into_dstring)
                ).last().unwrap(); // We checked the len
                self.lines[min.0].truncate(min.1);
                unsafe {
                    self.lines[min.0].push_slice(first_new_line.as_ref());
                    self.lines[min.0 + new_text_lines_n - 1].push_slice(&last_old_line[max.1..]);
                } // Safety: The caller
            }
        }

        unsafe {
            let pos_ptr = carets.cursors.carets[carets.cursor].get_position_mut_unchecked();
            if !pos_ptr.is_selection_none() {
                pos_ptr.set_selection_none();
                pos_ptr.set_cursor_unchecked(Cursor::new(max.0, max.1));
            }
        }

        carets.move_anything_after_ud_np_included(
            max.0,
            max.1,
            new_text_lines_n.saturating_sub(1).overflowing_sub(selected_text_lines_n.saturating_sub(1)).0 as isize,
            (last_new_line_len + if new_text_lines_n <= 1 { min.1 } else { 0 }).overflowing_sub(max.1).0 as isize,
        );
    }

    pub(crate) fn get_lines(&self) -> &Vec<DisplayString> {
        &self.lines
    }
    

    /// Safety: Make sure the encoding is correct
    pub(crate) unsafe fn get_selected_texts_to_copy(&self, cursors: &Carets) -> MostlyOneVec<MostlyOneVec<LittleString>> {
        (0..cursors.carets.len()).map(|i| {
            let min = cursors.carets[i].get_position().get_min();
            let max = cursors.carets[i].get_position().get_max(false);
            let this_cursor = (min.0..=max.0).map(|j| {
                let part = match (j == min.0, j == max.0) {
                    (true, true) => LittleString::from_slice(&self.lines[j][min.1..max.1]),
                    (true, false) => LittleString::from_slice(&self.lines[j][min.1..]),
                    (false, true) => LittleString::from_slice(&self.lines[j][..max.1]),
                    (false, false) => LittleString::from_slice(&self.lines[j]),
                };
                part
            }).collect::<MostlyOneVec<_>>();
            this_cursor
        }).collect()
    }
}