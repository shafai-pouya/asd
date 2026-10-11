#[cfg(feature = "debug-tools")]
pub mod inner {
    use crate::assets::colors::C_LOG_ERROR;
    use crate::backend::buffers::BuffersLock;
    use crate::backend::caret::{Carets, CursorEditor};
    use crate::backend::checkpoint::Checkpoints;
    use crate::backend::content::Content;
    use crate::backend::display_char::CharStyle;
    use crate::backend::display_string::DisplayString;
    use crate::backend::little_string::LittleString;
    use crate::edit_operators::EditOperators;
    use crate::ui::log::{LOGS, Log};
    use crate::{App, movec};
    use crossterm::event::{
        DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers,
    };
    use crossterm::execute;
    use crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };
    use once_cell::sync::Lazy;
    use ratatui::backend::Backend;
    use ratatui::buffer::Buffer;
    use ratatui::style::Color;
    use ratatui::{Frame, Terminal};
    use std::io;
    use std::io::Write;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use unicode_segmentation::UnicodeSegmentation;

    pub static DEBUG_TOOLS: DebugTools = DebugTools {
        activated: AtomicBool::new(false),
        commit_counts: AtomicUsize::new(0),
        real_commit_counts: AtomicUsize::new(0),
        last_commit_details: Lazy::new(|| Mutex::new(None)),
        needs_full_redraw: AtomicBool::new(false),
    };

    pub struct DebugTools {
        activated: AtomicBool,
        commit_counts: AtomicUsize,
        real_commit_counts: AtomicUsize,
        last_commit_details: Lazy<Mutex<Option<String>>>,
        needs_full_redraw: AtomicBool,
    }

    pub fn debug_key_events(event: KeyEvent) -> bool {
        if event.code == KeyCode::F(3) {
            match event.modifiers {
                KeyModifiers::NONE => {
                    DEBUG_TOOLS.activated.fetch_not(Ordering::Relaxed);
                    true
                }
                KeyModifiers::SHIFT => {
                    prompt_command();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn prompt_command() {
        let mut stdout = io::stdout();

        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();
        disable_raw_mode().unwrap();

        print!("Enter Command:");
        stdout.flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        DEBUG_TOOLS.needs_full_redraw.store(true, Ordering::Relaxed);

        let input = input.trim_end_matches(['\r', '\n']);

        'a: {
            if let Some(rest) = input.strip_prefix('r')
                && let Some((style, text)) = rest.split_once(':')
                && let Ok(style) = style.parse()
            {
                if let Some(style) = CharStyle::from_debug_number(style) {
                    replace_with_style(text, style);
                } else {
                    LOGS.push(Log {
                        message: "Invalid Style".to_string(),
                        color: C_LOG_ERROR,
                        handler: None,
                    })
                }
                break 'a;
            }
        }
        LOGS.push(Log {
            message: "Invalid command".to_string(),
            color: C_LOG_ERROR,
            handler: None,
        });
    }

    pub fn replace_with_style(text: &str, style: CharStyle) {
        let buffers = BuffersLock.get_debug_guard();
        let mut buffers = buffers.lock();
        let buffers = buffers.inner_mut();
        let buffer = buffers.active_mut();
        buffer.op_materialize_virtual_spaces();
        for i in 0..buffer.carets.carets.len() {
            unsafe {
                buffer.content.replace_text(
                    &mut buffer.checkpoints,
                    &mut CursorEditor {
                        cursor: i,
                        cursors: &mut buffer.carets,
                    },
                    movec!(LittleString::Big(DisplayString::from_str_utf8(text, style))),
                );
            }
        }
    }

    pub fn debug_tools_event_loop<B: Backend>(terminal: &mut Terminal<B>) {
        if DEBUG_TOOLS.needs_full_redraw.swap(false, Ordering::Relaxed) {
            terminal.clear().unwrap();
        }
    }

    pub fn debug_commit(_: &mut Checkpoints, _: &mut Carets, _: &Content) {
        DEBUG_TOOLS
            .real_commit_counts
            .fetch_add(1, Ordering::Relaxed);
    }
    pub fn debug_push_commit() {
        DEBUG_TOOLS.commit_counts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn debug_render(frame: &mut Frame, _: &mut App) {
        if !DEBUG_TOOLS.activated.load(Ordering::Relaxed) {
            return;
        }

        let buffer = frame.buffer_mut();
        fn print(buffer: &mut Buffer, line: u16, col: u16, string: String) {
            for (i, g) in string.graphemes(true).enumerate() {
                let i = i as u16;
                buffer[(col + i, line)].set_symbol(g);
                buffer[(col + i, line)].bg = Color::Blue;
                buffer[(col + i, line)].fg = Color::Reset;
            }
        }

        let debug_guard = BuffersLock.get_debug_guard();
        let mut debug_guard = debug_guard.lock();
        let debug_guard = debug_guard.inner_mut();
        let active = debug_guard.active();

        let checkpoints_len = active.checkpoints.others.len();
        let checkpoints_pos = active.checkpoints.cursor_lened;

        let mut line = 5;

        print(
            buffer,
            line,
            0,
            format!("Checkpoints: L{checkpoints_len} T{checkpoints_pos}"),
        );
        line += 1;

        let details_lock = DEBUG_TOOLS.last_commit_details.lock().unwrap();
        let details = if let Some(details) = &*details_lock {
            details
        } else {
            ""
        };
        print(
            buffer,
            line,
            0,
            format!(
                "Commits: C{}({}) {details}",
                DEBUG_TOOLS.commit_counts.load(Ordering::Relaxed),
                DEBUG_TOOLS.real_commit_counts.load(Ordering::Relaxed)
            ),
        );
        line += 2;

        print(buffer, line, 0, "Carets:".to_string());
        line += 1;

        for idx in 0..active.carets.carets.len() {
            let p = active.carets.carets[idx].get_position();
            let s1 = format!(
                "  {}:{}@{} ",
                p.cursor.line, p.cursor.col.col, p.cursor.col.goal
            );
            let len = s1.len() as u16;
            let s2 = format!(
                "{}{}:{}",
                if p.selection.is_none() { "N" } else { "S" },
                p.selection.line,
                p.selection.col
            );
            let len_2 = s2.len() as u16;
            let s3 = format!(
                " +{} -{:?}",
                active.carets.carets[idx].added_len,
                active.carets.carets[idx]
                    .removed_text
                    .map(|a| a.clone().into_dstring().to_string_utf8(active.encoding))
            );
            print(buffer, line, 0, s1);
            print(buffer, line, len, s2);
            print(buffer, line, len + len_2, s3);
            if p.selection.is_none() {
                for c in len..len + len_2 {
                    buffer[(c, line)].fg = Color::Black;
                }
            }
            line += 1;
        }
    }
}

#[cfg(not(feature = "debug-tools"))]
pub mod inner {
    use crate::App;
    use crate::backend::caret::Carets;
    use crate::backend::checkpoint::Checkpoints;
    use crate::backend::content::Content;
    use crossterm::event::KeyEvent;
    use ratatui::backend::Backend;
    use ratatui::{Frame, Terminal};

    #[inline(always)]
    pub fn debug_key_events(_: KeyEvent) -> bool {
        false
    }

    #[inline(always)]
    pub fn debug_render(_: &mut Frame, _: &mut App) {}

    #[inline(always)]
    pub fn debug_commit(_: &mut Checkpoints, _: &mut Carets, _: &Content) {}

    #[inline(always)]
    pub fn debug_push_commit() {}

    #[inline(always)]
    pub fn debug_tools_event_loop<B: Backend>(_: &mut Terminal<B>) {}
}
