#[cfg(feature = "debug-tools")]
pub mod inner {
    use crate::App;
    use crate::backend::buffers::BuffersLock;
    use crate::backend::caret::Carets;
    use crate::backend::checkpoint::Checkpoints;
    use crate::backend::content::Content;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use once_cell::sync::Lazy;
    use ratatui::Frame;
    use ratatui::buffer::Buffer;
    use ratatui::style::Color;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use unicode_segmentation::UnicodeSegmentation;

    pub static DEBUG_TOOLS: DebugTools = DebugTools {
        activated: AtomicBool::new(false),
        commit_counts: AtomicUsize::new(0),
        real_commit_counts: AtomicUsize::new(0),
        last_commit_details: Lazy::new(|| Mutex::new(None)),
    };

    pub struct DebugTools {
        activated: AtomicBool,
        commit_counts: AtomicUsize,
        real_commit_counts: AtomicUsize,
        last_commit_details: Lazy<Mutex<Option<String>>>,
    }

    pub fn debug_key_events(event: KeyEvent) -> bool {
        if event.modifiers == KeyModifiers::empty() && event.code == KeyCode::F(3) {
            DEBUG_TOOLS.activated.fetch_not(Ordering::Relaxed);
            true
        } else {
            false
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
    use ratatui::Frame;

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
}
