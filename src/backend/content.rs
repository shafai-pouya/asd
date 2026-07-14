use crate::backend::caret::{Carets, CursorEditor};
use crate::backend::checkpoint::checkpoints::{Checkpoints, DURATION_SMALL_TIMER};
use crate::backend::cursor::Cursor;
use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::MostlyOneVec;
use crate::ui::log::Log;
use libc::{access, W_OK, X_OK};
use std::ffi::CString;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::ops::{Index, Range};
use std::path::Path;
use std::time::Instant;
use crate::assets::colors::colors::{C_LOG_ERROR, C_LOG_INFO, C_LOG_WARNING};

#[derive(Default, Debug, Clone, Copy)]
pub enum LineEnding {
    CR,
    LF,
    #[default]
    CRLF,
}

impl LineEnding {
    pub(crate) fn get(&self) -> &[u8] {
        match self {
            LineEnding::CR => b"\r",
            LineEnding::LF => b"\n",
            LineEnding::CRLF => b"\r\n",
        }
    }
}


pub(crate) fn whitespaces_in_the_start_of_the_line(s: &str) -> &str {
    let end = s.find(|c: char| !c.is_whitespace())
        .unwrap_or(s.len());

    &s[..end]
}
pub struct Content {
    lines: Vec<String>,
}

impl Index<usize> for Content {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}


impl Content {
}

impl Content {
    pub(crate) fn from_file(path: &Path, logs: &mut Vec<Log>) -> (Self, LineEnding) {
        let content;
        // let mut logs = Vec::new();
        match fs::exists(path) {
            Err(e) => {
                logs.push(Log {
                    message: format!("[E:{}] Error checking existence of the file: {}", e.kind() as u32, e.kind().to_string()),
                    color: C_LOG_ERROR,
                });
                content = String::new();
            }
            Ok(false) => {
                logs.push(Log {
                    message: "[I] Created new file".to_string(), // todo: maybe couldn't
                    color: C_LOG_INFO,
                });
                content = String::new();
                {
                    let parent = CString::new(path.parent().unwrap().to_str().unwrap().to_string().as_bytes()).unwrap(); // Shouldn't fail I think
                    let result = unsafe { access(parent.as_ptr(), W_OK | X_OK) };
                    drop(parent);
                    if result != 0 {
                        logs.push(Log {
                            message: "[W] Warning: You will fail to save the file, I think...".to_string(), // todo: maybe couldn't
                            color: C_LOG_WARNING,
                        });
                    }
                }
            }
            Ok(true) => {
                match fs::read_to_string(path) {
                    Ok(c) => {
                        content = c;
                        match File::options().append(true).open(path) {
                            Ok(f) => drop(f),
                            Err(e) => {
                                logs.push(Log {
                                    message: format!("[E:{}] Error Opening File for in append mode: {}", e.kind() as u32, e.kind().to_string()),
                                    color: C_LOG_ERROR,
                                });
                                logs.push(Log {
                                    message: "[I] It means the file is readonly!".to_string(),
                                    color: C_LOG_INFO,
                                });
                            }
                        }
                    },
                    Err(e) => {
                        logs.push(Log {
                            message: format!("[E:{}] Error Opening File for the first time: {}", e.kind() as u32, e.kind().to_string()),
                            color: C_LOG_ERROR,
                        });
                        content = String::new();
                    }
                }
            }
        }
        let mut lines = vec![String::new()];
        let mut line_ending = None;
        let mut chars = content.chars().peekable().into_iter();
        while let Some(ch) = chars.next() {
            if ch == '\n' {
                line_ending.get_or_insert(LineEnding::LF);
                lines.push(String::new());
            } else if ch == '\r' {
                lines.push(String::new());
                if chars.peek() == Some(&'\n') {
                    chars.next();
                    line_ending.get_or_insert(LineEnding::CRLF);
                } else {
                    line_ending.get_or_insert(LineEnding::CR);
                }
            } else {
                lines.last_mut().unwrap().push(ch); // It's Ok
            }
        }

        (Self { lines }, line_ending.unwrap_or(LineEnding::CRLF))
    }

    #[inline]
    pub(crate) fn reserve_at_line(&mut self, line: usize, cap: usize) {
        self.lines[line].reserve(cap);
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }

    #[inline]
    pub(crate) fn get(&self, i: Range<usize>) -> Option<&[String]> {
        self.lines.get(i)
    }
    
    pub(crate) fn get_max_line_length(&self) -> usize {
        self.lines.iter().map(|l| l.len()).max().unwrap() // Ok
    }

    pub(crate) fn replace_text(&mut self, checkpoints: &mut Checkpoints, carets: &mut CursorEditor, new_text: &MostlyOneVec<LittleString>) {
        let caret_ptr = &mut carets.cursors.carets[carets.cursor];
        let new_text_lines_n = new_text.len();
        if caret_ptr.is_selection_none() {
            unsafe {
                let pos_ptr = caret_ptr.get_position_mut();
                pos_ptr.selection.line = pos_ptr.cursor.line;
            }
        }
        let pos_ptr = caret_ptr.get_position();
        let min = pos_ptr.get_min();
        let max = pos_ptr.get_max(false);

        // Do checkpoints:
        caret_ptr.start_checkpoint();
        let forward = caret_ptr.get_position().selection.get_lc() > caret_ptr.get_position().cursor.get_lc();
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
                for ch in part.as_bytes() {
                    if skip > 0 { skip -= 1; } else {
                        caret_ptr.removed_text.last_mut().unwrap().push(*ch as char); // It's Ok
                    }
                }
            } else {
                for ch in part.as_bytes().iter().rev() {
                    if skip > 0 { skip -= 1; } else {
                        caret_ptr.removed_text[0].insert(0, *ch);
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

        // // todo: remove this line:
        // let mut f = File::options().write(true).open("/dev/tty2").unwrap();
        // write!(f, "Added: {}\nRemoved: {:?}\n", carets.cursors.carets[carets.cursor].added_len, carets.cursors.carets[carets.cursor].removed_text).unwrap();
    }

    pub(crate) unsafe fn replace_text_without_checkpoints(&mut self, carets: &mut CursorEditor, new_text: &MostlyOneVec<LittleString>) {
        let caret_ptr = &mut carets.cursors.carets[carets.cursor];
        if caret_ptr.is_selection_none() {
            unsafe {
                let pos_ptr = caret_ptr.get_position_mut();
                pos_ptr.selection.line = pos_ptr.cursor.line;
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
                self.lines[pos_ptr.cursor.line].replace_range(
                    min.1..max.1,
                    new_text.get(0).map(LittleString::as_str).unwrap_or("")
                );
            }
            (0, _) |
            (1, _) => {
                let last_line = self.lines
                    .drain(min.0 + 1..max.0 + 1).last().unwrap(); // We checked the len in the match case

                self.lines[min.0].truncate(min.1);
                self.lines[min.0].push_str(new_text.get(0).map(LittleString::as_str).unwrap_or(""));
                self.lines[min.0].push_str(&last_line[max.1..]);
            }
            (_, 0) |
            (_, 1) => {
                let mut new_text = new_text.iter();
                let first_new_line = new_text.next().unwrap(); // We checked the length in the match case
                self.lines.splice(
                    min.0+1..min.0+1,
                    new_text.map(LittleString::to_string_clone)
                );
                let dst = min.0 + new_text_lines_n - 1;
                let (left, right) = self.lines.split_at_mut(dst);
                right[0].push_str(&left[min.0][min.1..]);
                self.lines[min.0].truncate(min.1);
                self.lines[min.0].push_str(first_new_line.as_str());
            }
            (_, _) => {
                let last_line = self.lines.splice(
                    min.0 + 1..max.0 + 1,
                    new_text.iter().skip(1).map(LittleString::to_string_clone)
                ).last().unwrap(); // We checked the len
                self.lines[min.0].truncate(min.1);
                self.lines[min.0].push_str(new_text[0].as_str());
                self.lines[min.0 + new_text_lines_n - 1].push_str(&last_line[max.1..]);
            }
        }

        unsafe {
            let pos_ptr = carets.cursors.carets[carets.cursor].get_position_mut();
            if pos_ptr.selection.none == false {
                pos_ptr.cursor = Cursor::new(max.0, max.1);
                pos_ptr.selection.none = true;
            }
        }

        carets.move_anything_after_ud_np_included(
            max.0,
            max.1,
            new_text_lines_n.saturating_sub(1).overflowing_sub(selected_text_lines_n.saturating_sub(1)).0 as isize,
            (last_new_line_len + if new_text_lines_n <= 1 { min.1 } else { 0 }).overflowing_sub(max.1).0 as isize,
        );
    }

    pub(crate) fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }

    pub(crate) fn get_selected_texts(&self, cursors: &Carets, line_ending: LineEnding) -> Vec<String> {
        let mut texts = vec![];
        for i in 0..cursors.carets.len() {
            let min = cursors.carets[i].get_position().get_min();
            let max = cursors.carets[i].get_position().get_max(false);
            let mut text = String::new();
            if min.0 == max.0 {
                text.push_str(&self.lines[min.0][min.1..max.1]);
            } else {
                text.push_str(&self.lines[min.0][min.1..]);
                text.push_str(&str::from_utf8(line_ending.get()).unwrap());
                for line in min.0 + 1..max.0 {
                    text.push_str(&self.lines[line]);
                    text.push_str(&str::from_utf8(line_ending.get()).unwrap());
                }
                text.push_str(&self.lines[max.0][..max.1]);
            }
            texts.push(text);
        }
        texts
    }

    pub(crate) fn get_copyable_text(&self, cursors: &Carets, line_ending: LineEnding) -> String {
        let texts = self.get_selected_texts(cursors, line_ending);
        if !(1..texts.len()).all(|i| texts[i] == texts[i - 1]) {
            let mut joined_text = String::with_capacity(
                line_ending.get().len() * (texts.len() - 1)
                + texts.iter().map(|t| t.len()).sum::<usize>()
            );
            let mut iter = texts.into_iter();
            joined_text.push_str(&iter.next().unwrap()); // There is at least one line
            for s in iter {
                joined_text.push_str(str::from_utf8(line_ending.get()).unwrap());
                joined_text.push_str(&s);
            }
            joined_text
        } else {
            texts.into_iter().next().unwrap_or(String::new())
        }
    }

}

impl Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineEnding::CR => write!(f, "CR"),
            LineEnding::LF => write!(f, "LF"),
            LineEnding::CRLF => write!(f, "CRLF"),
        }
    }
}
