use crate::App;
use crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::Widget;
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct Log {
    pub message: String,
    pub color: Color,
    pub handler: Option<fn(MouseEvent, &mut App)>,
}

pub(crate) fn render_logs(app: &App, frame: &mut Frame, logs_area: Rect) {
    let _ = app.logs.iter()
        .enumerate()
        .map(|(i, log)| {
            let mut x = log.message.clone()
                .fg(log.color);
            if log.handler.is_some() {
                x = x.underlined();
            }
            x
                .render(
                    Rect {
                        height: 1,
                        y: logs_area.y + i as u16,
                        ..logs_area
                    },
                    frame.buffer_mut()
                )
        })
        .collect::<Vec<_>>();
}

#[inline]
pub(crate) fn get_logs_height(app: &App) -> u16 {
    app.logs.len() as u16
}