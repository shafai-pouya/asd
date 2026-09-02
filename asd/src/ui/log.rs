use crate::App;
use crossterm::event::MouseEvent;
use once_cell::sync::Lazy;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::Widget;
use std::sync::Mutex;

static LOGS_LOCK: Lazy<Mutex<Vec<Log>>> = Lazy::new(|| Mutex::new(vec![]));

pub type LogEvent = fn(MouseEvent, &mut App);

pub struct LogsType;
pub static LOGS: LogsType = LogsType;

impl LogsType {
    pub fn push(&self, log: Log) {
        LOGS_LOCK.lock().unwrap().push(log);
    }
    pub fn clear(&self) {
        LOGS_LOCK.lock().unwrap().clear();
    }
    pub fn len(&self) -> usize {
        LOGS_LOCK.lock().unwrap().len()
    }
    pub fn get_clone(&self, index: usize) -> Option<Log> {
        LOGS_LOCK.lock().unwrap().get(index).cloned()
    }
    pub fn handler_of_mouse_event(
        &self,
        me: MouseEvent,
        last_content_rect: Rect,
    ) -> Option<LogEvent> {
        let logs_lock = LOGS_LOCK.lock().unwrap();
        if me.row > last_content_rect.height + 1 - logs_lock.len() as u16
            && let Some(log) = logs_lock
                .get(me.row as usize + logs_lock.len() - 2 - last_content_rect.height as usize)
        {
            return log.handler;
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub message: String,
    pub color: Color,
    pub handler: Option<LogEvent>,
}

pub(crate) fn render_logs(frame: &mut Frame, logs_area: Rect) {
    let _ = LOGS_LOCK
        .lock()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let mut x = log.message.clone().fg(log.color);
            if log.handler.is_some() {
                x = x.underlined();
            }
            x.render(
                Rect {
                    height: 1,
                    y: logs_area.y + i as u16,
                    ..logs_area
                },
                frame.buffer_mut(),
            )
        })
        .collect::<Vec<_>>();
}

#[inline]
pub(crate) fn get_logs_height() -> u16 {
    LOGS_LOCK.lock().unwrap().len() as u16
}
