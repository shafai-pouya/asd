use crate::assets::colors::{C_LOG_ERROR, C_LOG_INFO, C_LOG_WARNING};
use crate::backend::content::Content;
use crate::backend::display_char::{CharStyle, CharValue};
use crate::backend::display_string::DisplayString;
use crate::ui::log::{LOGS, Log};
use libc::{W_OK, X_OK, access};
use std::ffi::CString;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

#[derive(Default, Debug, Clone, Copy)]
pub enum LineEnding {
    Cr,
    Lf,
    #[default]
    Crlf,
}

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    UTF8(LineEnding),
    Raw,
}

impl LineEnding {
    pub(crate) fn get(&self) -> &[u8] {
        match self {
            LineEnding::Cr => b"\r",
            LineEnding::Lf => b"\n",
            LineEnding::Crlf => b"\r\n",
        }
    }
}

impl Encoding {
    pub(crate) fn save_buffer(&self, content: &Content, file_path: &Path) -> bool {
        match File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)
        {
            Ok(mut file) => {
                match self {
                    Encoding::UTF8(ending) => {
                        let mut first_line = true;
                        for line in content.get_lines() {
                            if !first_line {
                                match file.write_all(ending.get()) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        LOGS.push(Log {
                                            message: format!(
                                                "[E:{}] Error writing to file: {}",
                                                e.kind() as u32,
                                                e.kind()
                                            ),
                                            color: C_LOG_ERROR,
                                            handler: None,
                                        });
                                        return false;
                                    }
                                }
                            }
                            first_line = false;
                            match unsafe { line.utf8__write_to(&mut file) } {
                                // Safety: We know it's utf8
                                Ok(_) => {}
                                Err(e) => {
                                    LOGS.push(Log {
                                        message: format!(
                                            "[E:{}] Error writing to file: {}",
                                            e.kind() as u32,
                                            e.kind()
                                        ),
                                        color: C_LOG_ERROR,
                                        handler: None,
                                    });
                                    return false;
                                }
                            }
                        }
                    }
                    Encoding::Raw => {
                        for line in content.get_lines() {
                            match unsafe { line.raw__write_to(&mut file) } {
                                Ok(_) => {}
                                Err(e) => {
                                    LOGS.push(Log {
                                        message: format!(
                                            "[E:{}] Error writing to file: {}",
                                            e.kind() as u32,
                                            e.kind()
                                        ),
                                        color: C_LOG_ERROR,
                                        handler: None,
                                    });
                                    return false;
                                }
                            }
                        }
                    }
                }

                LOGS.push(Log {
                    message: "[I:1] File Saved!".to_string(),
                    color: Default::default(),
                    handler: None,
                });
                true
            }
            Err(e) => {
                LOGS.push(Log {
                    message: format!(
                        "[E:{}] Error opening file to save: {}",
                        e.kind() as u32,
                        e.kind()
                    ),
                    color: C_LOG_ERROR,
                    handler: None,
                });
                false
            }
        }
    }

    pub(crate) fn from_file(path: &Path) -> (Encoding, Content) {
        let content;
        // let mut logs = Vec::new();
        match fs::exists(path) {
            Err(e) => {
                LOGS.push(Log {
                    message: format!(
                        "[E:{}] Error checking existence of the file: {}",
                        e.kind() as u32,
                        e.kind()
                    ),
                    color: C_LOG_ERROR,
                    handler: None,
                });
                content = String::new();
            }
            Ok(false) => {
                LOGS.push(Log {
                    message: "[I] Created new file".to_string(), // todo: maybe couldn't
                    color: C_LOG_INFO,
                    handler: None,
                });
                content = String::new();
                {
                    let parent = CString::new(
                        path.parent()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                            .as_bytes(),
                    )
                    .unwrap(); // Shouldn't fail I think
                    let result = unsafe { access(parent.as_ptr(), W_OK | X_OK) };
                    drop(parent);
                    if result != 0 {
                        LOGS.push(Log {
                            message: "[W] Warning: You will fail to save the file, I think..."
                                .to_string(), // todo: maybe couldn't
                            color: C_LOG_WARNING,
                            handler: None,
                        });
                    }
                }
            }
            Ok(true) => match fs::read_to_string(path) {
                Ok(c) => {
                    content = c;
                    match File::options().append(true).open(path) {
                        Ok(f) => drop(f),
                        Err(e) => {
                            LOGS.push(Log {
                                message: format!(
                                    "[E:{}] Error Opening File for in append mode: {}",
                                    e.kind() as u32,
                                    e.kind()
                                ),
                                color: C_LOG_ERROR,
                                handler: None,
                            });
                            LOGS.push(Log {
                                message: "[I] It means the file is readonly!".to_string(),
                                color: C_LOG_INFO,
                                handler: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == ErrorKind::InvalidData {
                        return Self::from_file_raw(path);
                    } else {
                        LOGS.push(Log {
                            message: format!(
                                "[E:{}] Error Opening File for the first time: {}",
                                e.kind() as u32,
                                e.kind()
                            ),
                            color: C_LOG_ERROR,
                            handler: None,
                        });
                        content = String::new();
                    }
                }
            },
        }
        Self::from_str_utf8(&content)
    }

    pub(crate) fn from_str_utf8(content: &str) -> (Encoding, Content) {
        let mut lines: Vec<DisplayString> = vec![];
        let mut last = String::new();
        let mut line_ending = None;
        let mut chars = content.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\n' {
                line_ending.get_or_insert(LineEnding::Lf);
                lines.push(DisplayString::from_str(
                    &std::mem::take(&mut last),
                    CharStyle::NONE,
                ));
            } else if ch == '\r' {
                lines.push(DisplayString::from_str(
                    &std::mem::take(&mut last),
                    CharStyle::NONE,
                ));
                if chars.peek() == Some(&'\n') {
                    chars.next();
                    line_ending.get_or_insert(LineEnding::Crlf);
                } else {
                    line_ending.get_or_insert(LineEnding::Cr);
                }
            } else {
                last.push(ch); // It's Ok
            }
        }
        lines.push(DisplayString::from_str(&last, CharStyle::NONE));
        (
            Encoding::UTF8(line_ending.unwrap_or(LineEnding::Crlf)),
            Content::from_lines(lines),
        )
    }

    pub(crate) fn from_file_raw(path: &Path) -> (Self, Content) {
        let mut file = File::options().read(true).open(path).unwrap(); // todo: remove unwrap
        let mut last = DisplayString::empty();
        let mut lines = vec![];
        let mut buf = [0; 4096];
        while let Ok(len) = file.read(&mut buf) {
            // todo: handle errors
            if len == 0 {
                break;
            }
            let mut iter = buf[..len].iter().peekable();
            while let Some(&b) = iter.next() {
                CharValue::from_u8(b as u32, &mut last, CharStyle::NONE);
                if b == b'\n' {
                    lines.push(std::mem::replace(&mut last, DisplayString::empty()));
                } else if b == b'\r' {
                    if iter.peek() == Some(&&b'\n') {
                        iter.next();
                        unsafe {
                            CharValue::from_lookup_idx(10, &mut last, CharStyle::NONE);
                        } // Safety: The encoding is correct, and also the [idx/4] is correct (10)
                    }
                    lines.push(std::mem::replace(&mut last, DisplayString::empty()));
                }
            }
        }
        lines.push(last);
        (Encoding::Raw, Content::from_lines(lines))
    }
}

impl Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineEnding::Cr => write!(f, "CR"),
            LineEnding::Lf => write!(f, "LF"),
            LineEnding::Crlf => write!(f, "CRLF"),
        }
    }
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoding::UTF8(ending) => {
                write!(f, "UTF8, ")?;
                Display::fmt(ending, f)
            }
            Encoding::Raw => write!(f, "Raw bytes"),
        }
    }
}
