use crate::assets::colors::C_LOG_INFO;
use crate::assets::constants::{N_DRAIN_CHECKPOINTS, N_MAX_CHECKPOINTS};
use crate::backend::caret::Carets;
use crate::backend::content::Content;
use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::MostlyOneVec;
use crate::movec;
use crate::ui::log::{LOGS, Log};
use std::time::Instant;

#[derive()]
pub(crate) struct Checkpoint {
    pub inner: MostlyOneVec<SingleEdit>,
}

#[derive(Clone)]
pub(crate) struct SingleEdit {
    pub edit: CheckpointEdit,
}

#[derive(Clone)]
pub(crate) struct CheckpointEdit {
    pub start_line: usize,
    pub start_col: usize,
    pub removed_data: MostlyOneVec<LittleString>,
    pub added_data: MostlyOneVec<LittleString>,
}

pub(crate) struct Checkpoints {
    pub others: Vec<Checkpoint>,
    pub cursor_lened: usize,
    pub little_timer_deadline: Option<Instant>,
    pub big_timer_deadline: Option<Instant>,
}

impl Checkpoints {
    pub fn drop_commit(&mut self, carets: &mut Carets) {
        self.little_timer_deadline = None;
        self.big_timer_deadline = None;
        for caret in carets {
            #[cfg(debug_assertions)]
            {
                caret.started = false;
            }
            caret.added_len = 0;
            caret.removed_text = movec!();
        }
    }
    pub(crate) fn commit(&mut self, carets: &mut Carets, content: &Content) {
        self.little_timer_deadline = None;
        self.big_timer_deadline = None;
        if carets.carets.iter().all(|c| {
            c.added_len == 0 && c.removed_text.get(0).map(|a| a.len() == 0).unwrap_or(true)
        }) {
            for caret in carets {
                #[cfg(debug_assertions)]
                {
                    caret.started = false;
                }
                caret.added_len = 0;
                caret.removed_text = movec!();
            }
            return;
        }
        self.push(Checkpoint {
            inner: carets
                .carets
                .iter_mut()
                .map(|caret| {
                    let mut line = caret.get_position().cursor.get_line();
                    let mut col = caret.get_position().cursor.get_col();
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
                    SingleEdit {
                        edit: CheckpointEdit {
                            start_line: line,
                            start_col: col,
                            removed_data: std::mem::replace(&mut caret.removed_text, movec!()),
                            added_data: result,
                        },
                    }
                })
                .collect(),
        })
    }
    fn push(&mut self, checkpoint: Checkpoint) {
        self.others.truncate(self.cursor_lened);
        self.others.push(checkpoint);
        self.cursor_lened += 1;
        if self.cursor_lened >= N_MAX_CHECKPOINTS {
            self.others.drain(..N_DRAIN_CHECKPOINTS);
            self.cursor_lened = self.others.len();
            LOGS.push(Log {
                message: "Removed some old checkpoints...".to_string(),
                color: C_LOG_INFO,
                handler: None,
            });
        }
    }
    pub(crate) fn new() -> Checkpoints {
        Checkpoints {
            cursor_lened: 0,
            others: Vec::new(),
            big_timer_deadline: None,
            little_timer_deadline: None,
        }
    }

    /// Ghost checkpoints used to do nothing
    pub fn ghost() -> Self {
        Checkpoints {
            cursor_lened: 0,
            others: Vec::new(),
            big_timer_deadline: None,
            little_timer_deadline: None,
        }
    }
}
