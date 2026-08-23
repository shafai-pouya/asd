use crate::assets::colors::colors::C_LOG_INFO;
use crate::assets::constants::{N_DRAIN_CHECKPOINTS, N_MAX_CHECKPOINTS};
use crate::backend::caret::Carets;
use crate::backend::checkpoint::checkpoint::{Checkpoint, SingleEdit};
use crate::backend::checkpoint::checkpoint_line::CheckpointEdit;
use crate::backend::content::Content;
use crate::backend::little_string::LittleString;
use crate::movec;
use crate::ui::log::Log;
use std::time::Instant;

pub(crate) struct Checkpoints {
    pub others: Vec<Checkpoint>,
    pub cursor_lened: usize,
    pub little_timer_deadline: Option<Instant>,
    pub big_timer_deadline: Option<Instant>,
}

impl Checkpoints {
    pub(crate) fn commit(&mut self, carets: &mut Carets, logs: &mut Vec<Log>, content: &Content) {
        self.little_timer_deadline = None;
        self.big_timer_deadline = None;
        if carets.carets.iter().any(|c| {
            c.added_len == 0 && c.removed_text.get(0).map(|a| a.len() == 0).unwrap_or(true)
        }) {
            return;
        }
        self.push(
            Checkpoint {
                inner: carets.carets.iter_mut()
                    .map(|caret| {
                        let mut line = caret.get_position().cursor().get_line();
                        let mut col = caret.get_position().cursor().get_col();
                        let mut result = movec!();

                        loop {
                            let available = col;

                            if caret.added_len <= available {
                                let start = available - caret.added_len;

                                let text = &content[line][start..start + caret.added_len];

                                result.insert(0, LittleString::from_slice(text));

                                col = start;
                                caret.added_len = 0;
                                break;
                            }

                            let text = &content[line][..available];

                            result.insert(0, LittleString::from_slice(text));

                            caret.added_len -= available;

                            if line == 0 {
                                col = 0;
                                break;
                            }

                            if caret.added_len == 0 {
                                col = 0;
                                break;
                            }

                            caret.added_len -= 1;
                            line -= 1;
                            col = content[line].len();
                        }
                        let edit = SingleEdit {
                            edit: CheckpointEdit {
                                start_line: line,
                                start_col: col,
                                removed_data: std::mem::replace(&mut caret.removed_text, movec!()),
                                added_data: result,
                            },
                        };
                        edit
                    })
                    .collect(),
            }, logs)
    }
}

impl Checkpoints {
    pub(crate) fn push(&mut self, checkpoint: Checkpoint, logs: &mut Vec<Log>) {
        self.others.truncate(self.cursor_lened);
        self.others.push(checkpoint);
        self.cursor_lened += 1;
        if self.cursor_lened >= N_MAX_CHECKPOINTS {
            self.others.drain(..N_DRAIN_CHECKPOINTS);
            self.cursor_lened = self.others.len();
            logs.push(Log {
                message: "Removed some old checkpoints...".to_string(),
                color: C_LOG_INFO,
                handler: None,
            });
        }
    }
}

impl Checkpoints {
    pub(crate) fn new() -> Checkpoints {
        Checkpoints {
            cursor_lened: 0,
            others: Vec::new(),
            big_timer_deadline: None,
            little_timer_deadline: None,
        }
    }
}