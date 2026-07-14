use crate::backend::caret::Carets;
use crate::backend::content::Content;
use crate::ui::cursor::TerminalCursor;
use crate::ui::custom_scrollbar::CustomScrollbar;
use crate::App;
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use std::cmp::Ordering;
use crate::assets::colors::colors::{C_BG_CURSOR, C_BG_CURSOR_SELECTION, C_BG_SELECTION, C_FG_CURSOR, C_FG_CURSOR_SELECTION, C_FG_LINE_NUMBERS, C_FG_SELECTION};

pub(crate) fn render_file(app: &mut App, file_area: Rect, file_scroll_area: Rect, buf: &mut Buffer, can_use_cursor: bool) -> Rect {
    // Layout split
    let layout = Layout::horizontal([
        Constraint::Length(6),
        Constraint::Min(0),
    ]);
    let [lines_area, content_area] = file_area.layout(&layout);

    // Set bg/fg
    Block::default()
        .fg(C_FG_LINE_NUMBERS)
        .render(lines_area, buf);

    // Other logic
    let active_buffer = app.buffers.active_mut();
    active_buffer.scrollbar.validate_position(&active_buffer.content, content_area);

    Paragraph::new(
        active_buffer.content.get(
            active_buffer.scrollbar.top_position..
                (active_buffer.scrollbar.top_position+content_area.height as usize).min(active_buffer.content.len()))
            .unwrap_or(&[]).iter()
            .map(|a| Line::raw(
                a.get(
                    active_buffer.scrollbar.position as usize..
                        ((active_buffer.scrollbar.position + content_area.width) as usize).min(a.len())
                ).unwrap_or(""))).collect::<Vec<_>>()
    )
        .render(content_area, buf);

    Paragraph::new(
        (
            active_buffer.scrollbar.top_position+1..
                (active_buffer.scrollbar.top_position+content_area.height as usize+1).min(active_buffer.content.len() + 1)
        )
            .map(|n| Line::raw(format!("{} ", n.to_string()))
                .alignment(Alignment::Right))
            .collect::<Vec<_>>()
    )
        .render(lines_area, buf);

    active_buffer.scrollbar.render(file_scroll_area, content_area, buf, &active_buffer.content);

    active_buffer.carets.merge(); // todo: I think need to delete this

    render_cursor(
        &active_buffer.carets,
        &active_buffer.content,
        &active_buffer.scrollbar,
        content_area, 
        &mut app.terminal_cursor,
        buf, 
        can_use_cursor
    );

    content_area
}

pub(crate) fn render_cursor(cursors: &Carets, content: &Content, scrollbar: &CustomScrollbar, content_area: Rect, terminal_cursor: &mut TerminalCursor, buf: &mut Buffer, can_use_cursor: bool) {
    let len = cursors.carets.len();
    if can_use_cursor && len == 1 && cursors.carets[0].get_position().selection.is_none() {
        if let Ok((x, y)) = find_in_viewport_position(cursors.carets[0].get_position().cursor.line, cursors.carets[0].get_position().cursor.col, content_area, scrollbar) {
            terminal_cursor.set_to((x, y));
        } else {
            terminal_cursor.hide();
        }
    } else {
        if can_use_cursor {
            terminal_cursor.hide();
        }
        for cert in &cursors.carets {
            let pos = cert.get_position();
            if pos.selection.is_none() {
                if let Ok((x, y)) = find_in_viewport_position(
                    pos.cursor.line,
                    pos.cursor.col,
                    content_area,
                    scrollbar,
                ) {
                    Block::new()
                        .bg(C_BG_CURSOR)
                        .fg(C_FG_CURSOR)
                        .render(
                            Rect {
                                x,
                                y,
                                width: 1,
                                height: 1,
                            },
                            buf
                        )
                }
            }
            else {
                #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
                struct LC { line: usize, col: usize }
                let mut start;
                let mut end;
                let cursor = LC { line: pos.cursor.line, col: pos.cursor.col };
                let selection = LC { line: pos.selection.line, col: pos.selection.col };
                if cursor >= selection {
                    start = selection;
                    end = cursor;
                } else {
                    start = cursor;
                    end = selection;
                }
                let min_line = content_area.y as usize + scrollbar.top_position;
                if start.line < min_line {
                    start = LC { line: min_line, col: 0 };
                }
                let max_line = min_line + content_area.height as usize - 1;
                if end.line > max_line {
                    end = LC { line: max_line, col: content[max_line].len() };
                }

                match start.line.cmp(&end.line) {
                    Ordering::Less => {
                        let (x1, _) = find_in_viewport_position(start.line, start.col, content_area, scrollbar).unwrap_or_else(|a| a);
                        let (x2, y1) = find_in_viewport_position(start.line, content[start.line].len(), content_area, scrollbar).unwrap_or_else(|a| a);
                        Block::new()
                            .bg(C_BG_SELECTION)
                            .fg(C_FG_SELECTION)
                            .render(Rect { x: x1, y: y1, width: x2 - x1, height: 1, }, buf);

                        for line in start.line + 1..end.line {
                            let (x1, _) = find_in_viewport_position(line, 0, content_area, scrollbar).unwrap_or_else(|a| a);
                            let (x2, y1) = find_in_viewport_position(line, content[line].len(), content_area, scrollbar).unwrap_or_else(|a| a);
                            Block::new()
                                .bg(C_BG_SELECTION)
                                .fg(C_FG_SELECTION)
                                .render(Rect { x: x1, y: y1, width: x2 - x1, height: 1, }, buf);
                        }

                        let (x1, _) = find_in_viewport_position(end.line, 0, content_area, scrollbar).unwrap_or_else(|a| a);
                        let (x2, y1) = find_in_viewport_position(end.line, end.col, content_area, scrollbar).unwrap_or_else(|a| a);
                        Block::new()
                            .bg(C_BG_SELECTION)
                            .fg(C_FG_SELECTION)
                            .render(Rect { x: x1, y: y1, width: x2-x1, height: 1, }, buf);
                    }
                    Ordering::Equal => {
                        let (x1, _) = find_in_viewport_position(start.line, start.col, content_area, scrollbar).unwrap_or_else(|a| a);
                        let (x2, y1) = find_in_viewport_position(start.line, end.col, content_area, scrollbar).unwrap_or_else(|a| a);
                        Block::new()
                            .bg(C_BG_SELECTION)
                            .fg(C_FG_SELECTION)
                            .render(Rect { x: x1, y: y1, width: x2.overflowing_sub(x1).0, height: 1, }, buf);

                    }
                    Ordering::Greater => {}
                }
                if let Ok((x, y)) = find_in_viewport_position(cursor.line, cursor.col, content_area, scrollbar) {
                    // Note: I don't prefer:
                    //   if cursor < selection {
                    //       x -= 1;
                    //   }
                    Block::new()
                        .bg(C_BG_CURSOR_SELECTION)
                        .fg(C_FG_CURSOR_SELECTION)
                        .render(Rect { x, y, width: 1, height: 1, }, buf);
                }
            }
        }
    }
}

pub(crate) fn find_in_viewport_position(
    line: usize,
    col: usize,
    content_area: Rect,
    scrollbar: &CustomScrollbar,
) -> Result<(u16, u16), (u16, u16)> {
    let new_cursor_y = line.wrapping_sub(scrollbar.top_position) as u16;
    if new_cursor_y >= content_area.height {
        return Err((0, 0)); // Not important for us what is it
    }

    let new_cursor_x = col.wrapping_sub(scrollbar.position as usize) as u16;
    if new_cursor_x >= content_area.width {
        return Err((
            if (new_cursor_x as i16) < 0 {
                0
            } else {
                content_area.width
            } + content_area.x, new_cursor_y + content_area.y
        ));
    }

    Ok((new_cursor_x + content_area.x, new_cursor_y + content_area.y))
}