use crate::assets::colors::C_LOG_INFO;
use crate::assets::constants::DURATION_BIG_TIMER;
use crate::backend::caret::Carets;
use crate::backend::checkpoint::Checkpoints;
use crate::backend::content::Content;
use crate::backend::encoding::Encoding;
use crate::ui::custom_scrollbar::CustomScrollbar;
use crate::ui::log::{LOGS, Log};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) struct Buffer {
    pub path: PathBuf,
    pub showing_filename: String,

    pub carets: Carets,
    pub scrollbar: CustomScrollbar,
    pub drag_start_pos: (u16, u16),
    pub tab_size: usize,
    pub modified: bool,

    pub encoding: Encoding,
    pub content: Content,

    pub checkpoints: Checkpoints,
}

impl Buffer {
    pub(crate) fn new_from_file(path: PathBuf) -> Buffer {
        let showing_filename = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_str()
            .unwrap()
            .to_string();
        let (encoding, content) = Encoding::from_file(&path);
        Self::new(path, showing_filename, content, encoding)
    }
    pub(crate) fn new_utf8(path: PathBuf, showing_filename: String, content: &str) -> Self {
        let (encoding, content) = Encoding::from_str_utf8(content);
        Self::new(path, showing_filename, content, encoding)
    }

    pub(crate) fn new(
        path: PathBuf,
        showing_filename: String,
        content: Content,
        encoding: Encoding,
    ) -> Buffer {
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

    pub(crate) fn save(&mut self, file_path: Option<&Path>) {
        if self
            .encoding
            .save_buffer(&self.content, file_path.unwrap_or(&self.path))
        {
            self.modified = false;
        }
    }

    #[inline]
    pub(crate) fn commit(&mut self) {
        self.checkpoints.commit(&mut self.carets, &self.content);
    }

    #[inline]
    pub(crate) fn drop_commit(&mut self) {
        self.checkpoints.drop_commit(&mut self.carets);
    }

    pub(crate) fn buffer_modified(&mut self) {
        self.modified = true;
        if self.checkpoints.big_timer_deadline.is_none() {
            self.checkpoints.big_timer_deadline = Some(Instant::now() + DURATION_BIG_TIMER);
        }
    }

    pub(crate) fn try_quit(&self) -> Result<(), ()> {
        if self.modified {
            LOGS.push(Log {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_quit() {
        let mut buffer = Buffer::new_utf8(PathBuf::from("a"), "b".to_string(), "c");
        assert_eq!(buffer.try_quit(), Ok(()));

        buffer.buffer_modified();
        assert_eq!(buffer.try_quit(), Err(()));
    }

    #[test]
    fn save() {
        let mut buffer = Buffer::new_utf8(PathBuf::from("a"), "b".to_string(), "c");
        buffer.buffer_modified();

        buffer.save(Some(&PathBuf::from("/dev/null")));
        assert_eq!(buffer.try_quit(), Ok(()));
    }
}
