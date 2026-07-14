use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use crate::assets::colors::colors::{C_LOG_ERROR, C_LOG_INFO};
use crate::backend::caret::Carets;
use crate::backend::checkpoint::checkpoints::{Checkpoints, DURATION_BIG_TIMER};
use crate::backend::content::{Content, LineEnding};
use crate::ui::custom_scrollbar::CustomScrollbar;
use crate::ui::log::Log;

pub(crate) struct Buffer {
    pub path: PathBuf,
    pub showing_filename: String,
    
    pub carets: Carets,
    pub content: Content,
    pub scrollbar: CustomScrollbar,
    pub drag_start_pos: (usize, usize),
    pub tab_size: usize,
    pub modified: bool,
    pub line_ending: LineEnding,
    pub checkpoints: Checkpoints,
}

impl Buffer {
    pub(crate) fn new(path: PathBuf, logs: &mut Vec<Log>) -> Buffer {
        let showing_filename = path.file_name().unwrap_or(OsStr::new(path.as_os_str())).to_str().unwrap().to_string();
        let (content, line_ending) = Content::from_file(&path, logs);
        Buffer {
            path,
            showing_filename,
            carets: Carets::new(),
            content,
            scrollbar: CustomScrollbar::new(),
            drag_start_pos: (0, 0),
            tab_size: 4,
            modified: false,
            line_ending,
            checkpoints: Checkpoints::new(),
        }
    }

    
    pub(crate) fn save(&mut self, file_path: Option<&Path>, logs: &mut Vec<Log>) {
        match File::options().write(true).truncate(true).create(true).open(file_path.unwrap_or(&self.path)) {
            Ok(mut file) => {
                let mut first_line = true;
                for line in self.content.get_lines() {
                    if !first_line {
                        match file.write_all(self.line_ending.get()) {
                            Ok(_) => {},
                            Err(e) => {
                                logs.push(Log {
                                    message: format!("[E:{}] Error writing to file: {}", e.kind() as u32, e.kind().to_string()),
                                    color: C_LOG_ERROR,
                                });
                                return;
                            }
                        }
                    }
                    first_line = false;
                    match file.write_all(line.as_bytes()) {
                        Ok(_) => {},
                        Err(e) => {
                            logs.push(Log {
                                message: format!("[E:{}] Error writing to file: {}", e.kind() as u32, e.kind().to_string()),
                                color: C_LOG_ERROR,
                            });
                            return;
                        }
                    }
                }

                self.modified = false;

                logs.push(Log {
                    message: "[I:1] File Saved!".to_string(),
                    color: Default::default(),
                });
            }
            Err(e) => {
                logs.push(Log {
                    message: format!("[E:{}] Error opening file to save: {}", e.kind() as u32, e.kind().to_string()),
                    color: C_LOG_ERROR,
                });
            }
        }
    }
    
    #[inline]
    pub(crate) fn commit(&mut self, logs: &mut Vec<Log>) {
        self.checkpoints.commit(&mut self.carets, logs, &self.content);
    }


    pub(crate) fn buffer_modified(&mut self) {
        self.modified = true;
        if self.checkpoints.big_timer_deadline.is_none() {
            self.checkpoints.big_timer_deadline = Some(Instant::now() + DURATION_BIG_TIMER);
        }
    }

    pub(crate) fn try_quit(&mut self, logs: &mut Vec<Log>) -> Result<(), ()> {
        if self.modified {
            logs.push(Log {
                message: "Buffer is modified, try save it first".to_string(),
                color: C_LOG_INFO,
            });
            Err(())
        } else {
            Ok(())
        }
    }
}