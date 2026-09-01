use crate::assets::constants::{SCROLLBAR_EMPTY_COLS_PADDING, SCROLLBAR_EMPTY_ROWS_PADDING};
use crate::backend::content::Content;
use ratatui::layout::Rect;

pub struct CustomScrollbar {
    pub position: u16,
    pub top_position: usize,
    pub freeze: bool,
}

impl CustomScrollbar {
    pub(crate) fn new() -> Self {
        CustomScrollbar {
            position: 0,
            top_position: 0,
            freeze: false,
        }
    }

    pub(crate) fn get_start_end(
        &mut self,
        file_scroll_area: Rect,
        content_area: Rect,
        content: &Content,
    ) -> (u16, u16) {
        let content_width = content.get_max_line_length() + SCROLLBAR_EMPTY_COLS_PADDING;
        let viewport_width = content_area.width as usize;
        let scrollbar_width = file_scroll_area.width as usize;
        let visible_start = self.position as usize;

        if content_width <= viewport_width {
            (0, scrollbar_width as u16)
        } else {
            let max_scroll = content_width - viewport_width;

            let thumb_width = ((viewport_width as f64 / content_width as f64)
                * scrollbar_width as f64)
                .round()
                .max(1.0) as usize;

            let track_width = scrollbar_width.saturating_sub(thumb_width);

            let thumb_start = ((visible_start.min(max_scroll) as f64 / max_scroll as f64)
                * track_width as f64)
                .round() as usize;

            (
                thumb_start as u16,
                thumb_width.min(scrollbar_width - thumb_start) as u16,
            )
        }
    }

    /// Notes:
    /// - Remove messages
    pub(crate) fn prev(&mut self, i: u16) {
        self.position = self.position.saturating_sub(i);
    }

    /// Notes:
    /// - Remove messages
    pub(crate) fn next(&mut self, i: u16) {
        self.position += i;
    }

    /// Notes:
    /// - Remove messages
    pub(crate) fn prev_line(&mut self, i: usize) {
        self.top_position = self.top_position.saturating_sub(i);
    }

    /// Notes:
    /// - Remove messages
    pub(crate) fn next_line(&mut self, i: usize) {
        self.top_position += i;
    }

    pub(crate) fn validate_position(&mut self, content: &Content, content_area: Rect) {
        if let Some(max_position) = ((content.get_max_line_length() + SCROLLBAR_EMPTY_COLS_PADDING)
            as u16)
            .checked_sub(content_area.width)
        {
            self.position = self.position.min(max_position);
        } else {
            self.position = 0;
        }

        if let Some(max_position) =
            (content.len() + SCROLLBAR_EMPTY_ROWS_PADDING).checked_sub(content_area.height as usize)
        {
            self.top_position = self.top_position.min(max_position);
        } else {
            self.top_position = 0;
        }
    }

    pub(crate) fn ensure_cursor_visible(&mut self, x: usize, y: usize, last_content_rect: Rect) {
        if self.freeze {
            return;
        }

        if y < self.top_position {
            self.top_position = y;
        }

        if y >= self.top_position + last_content_rect.height as usize {
            self.top_position = y - last_content_rect.height as usize + 1;
        }

        if (x as u16) < self.position {
            self.position = x as u16
        }

        if x as u16 >= (self.position + last_content_rect.width) {
            self.position = x as u16 - last_content_rect.width + 1;
        }
    }
}
