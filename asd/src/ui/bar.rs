use crate::assets::colors::{C_BG_BAR, C_BG_NORMAL, C_FG_BAR};
use crate::backend::buffers::BuffersRenderGuard;
use chrono::Local;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Widget};

pub(crate) fn render_bar(
    bar: Rect,
    under_bar: Rect,
    buf: &mut Buffer,
    buffers: &mut BuffersRenderGuard,
) {
    Block::default().bg(C_BG_BAR).fg(C_FG_BAR).render(bar, buf);
    Block::default().bg(C_BG_NORMAL).render(under_bar, buf);

    let mut flags = String::new();

    let active_buffer = buffers.inner().active();

    if active_buffer.modified {
        flags.push_str(" [+]"); // todo: maybe will change it to all buffers?
    }

    if active_buffer.scrollbar.freeze {
        flags.push_str(" ⏸"); // todo: maybe will change it to all buffers?
    }

    let _ = (0..20 - flags.chars().count())
        .map(|_| flags.push(' '))
        .collect::<Vec<_>>();

    let left_str = format!(
        " {}{} {}",
        active_buffer.showing_filename, flags, active_buffer.carets
    ); // todo: maybe will change it to all buffers?

    let right_str = format!(
        "{}    {} ",
        active_buffer.encoding,
        Local::now().format("%H:%M")
    ); // todo: maybe will change it to all buffers?

    let horizontal = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(right_str.len() as u16),
    ]);

    let [left, right] = bar.layout(&horizontal);

    left_str.render(left, buf);
    right_str.render(right, buf);
}
