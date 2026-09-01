use crate::assets::colors::colors::C_FG_SCROLLBAR;
use crate::backend::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::style::Stylize;
use ratatui::widgets::Widget;

pub fn render_scrollbar(file_scroll_area: Rect, content_area: Rect, buf: &mut ratatui::buffer::Buffer, active_buffer: &mut Buffer) {
    let (scrollbar_start, scrollbar_width) =
        active_buffer.scrollbar.get_start_end(file_scroll_area, content_area, &active_buffer.content);

    let spans = vec![
        Span::raw("-".repeat(scrollbar_start as usize)),
        Span::raw("#".repeat(scrollbar_width as usize)),
        Span::raw("-".repeat((file_scroll_area.width - scrollbar_start - scrollbar_width) as usize)),
    ];

    Line::from(spans)
        .fg(C_FG_SCROLLBAR)
        .render(file_scroll_area, buf);
}