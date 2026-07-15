use crate::assets::colors::colors::C_LOG_INFO;
use crate::assets::constants::DURATION_BIG_TIMER;
use crate::backend::caret::Carets;
use crate::backend::checkpoint::checkpoints::Checkpoints;
use crate::backend::content::Content;
use crate::backend::encoding::Encoding;
use crate::ui::custom_scrollbar::CustomScrollbar;
use crate::ui::log::Log;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) struct Buffer {
    pub path: PathBuf,
    pub showing_filename: String,
    
    pub carets: Carets,
    pub scrollbar: CustomScrollbar,
    pub drag_start_pos: (usize, usize),
    pub tab_size: usize,
    pub modified: bool,

    pub encoding: Encoding,
    pub content: Content,

    pub checkpoints: Checkpoints,
}

impl Buffer {
    pub(crate) fn new_from_file(path: PathBuf, logs: &mut Vec<Log>) -> Buffer {
        let showing_filename = path.file_name().unwrap_or(OsStr::new(path.as_os_str())).to_str().unwrap().to_string();
        let (encoding, content) = Encoding::from_file(&path, logs);
        Self::new(path, showing_filename, content, encoding)
    }
    pub(crate) fn new_custom(path: PathBuf, showing_filename: String, content: &str) -> Self {
        let (encoding, content) = Encoding::from_str_utf8(content);
        Self::new(path, showing_filename, content, encoding)
    }
    
    pub(crate) fn new(path: PathBuf, showing_filename: String, content: Content, encoding: Encoding) -> Buffer {
        Buffer {
            encoding,
            path,
            showing_filename,
            carets: Carets::new(),
            content,
            scrollbar: CustomScrollbar::new(),
            drag_start_pos: (0, 0),
            tab_size: 4,
            modified: false,
            checkpoints: Checkpoints::new(),
        }
    }

    
    pub(crate) fn save(&mut self, file_path: Option<&Path>, logs: &mut Vec<Log>) {
        if self.encoding.save_buffer(&self.content, file_path.unwrap_or(&self.path), logs) {
            self.modified = false;
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
                handler: None,
            });
            Err(())
        } else {
            Ok(())
        }
    }
}