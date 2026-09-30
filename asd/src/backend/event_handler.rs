use crate::App;
use crate::assets::constants::DOUBLE_CLICK_DURATION;
use crate::debug_tools::inner::debug_key_events;
use crossterm::event::{Event, KeyEvent, MouseEvent, MouseEventKind};
use std::time::Instant;

pub type MouseHandlerFn<T> = fn(arg: &mut T, &EventHandler<T>, &mut App, &MouseEvent) -> bool;
pub type KeyHandlerFn<T> = fn(arg: &mut T, &mut App, &KeyEvent) -> bool;
pub type DoubleClickHandlerFn<T> = fn(arg: &mut T, &mut App, &MouseEvent) -> bool;

pub struct EventHandler<T> {
    key_handlers: Vec<KeyHandlerFn<T>>,
    mouse_handlers: Vec<MouseHandlerFn<T>>,
    double_click_handlers: Vec<DoubleClickHandlerFn<T>>,
}

impl<T> EventHandler<T> {
    pub(crate) fn new(
        key_handlers: Vec<KeyHandlerFn<T>>,
        mouse_handlers: Vec<MouseHandlerFn<T>>,
        double_click_handlers: Vec<DoubleClickHandlerFn<T>>,
    ) -> Self {
        Self {
            key_handlers,
            mouse_handlers,
            double_click_handlers,
        }
    }

    pub(crate) fn handle_event(&mut self, arg: &mut T, app: &mut App, event: Event) {
        match event {
            Event::Key(event) => self.handle_key_event(arg, app, event),
            Event::Mouse(event) => self.handle_mouse_event(arg, app, event),
            _ => {}
        }
    }

    fn handle_key_event(&mut self, arg: &mut T, app: &mut App, event: KeyEvent) {
        if debug_key_events(event) {
            return;
        }
        for key_handler in &self.key_handlers {
            if !key_handler(arg, app, &event) {
                break;
            }
        }
    }

    fn handle_mouse_event(&mut self, arg: &mut T, app: &mut App, event: MouseEvent) {
        for mouse_handler in &self.mouse_handlers {
            if !mouse_handler(arg, self, app, &event) {
                break;
            }
        }
    }

    pub(crate) fn default_double_click_handler(
        arg: &mut T,
        self_: &Self,
        app: &mut App,
        e: &MouseEvent,
    ) -> bool {
        if !matches!(e.kind, MouseEventKind::Down(_)) {
            return true;
        }

        let now = Instant::now();

        if (now - app.double_click_details.2 > DOUBLE_CLICK_DURATION)
            || (e.column != app.double_click_details.0)
            || (e.row != app.double_click_details.1)
        {
            app.double_click_details = (e.column, e.row, now);
        } else {
            for double_click_handler in &self_.double_click_handlers {
                if !double_click_handler(arg, app, e) {
                    return false;
                }
            }
        }
        true
    }
}
