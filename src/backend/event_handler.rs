use std::time::{Duration, Instant};
use bitflags::bitflags;
use crossterm::event::{Event, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use crate::App;

pub const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(300);

bitflags! {
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct EventFlags: u8 {
        // const MouseEvent = 1 << 0;
        const M_SHIFT = 1 << 1;
        const M_CTRL = 1 << 2;
        const M_ALT = 1 << 3;
        const M_SUPER = 1 << 4;
        // const DoubleClicked = 1 << 5;

        const AllModifiers = EventFlags::M_SHIFT.bits() |
            EventFlags::M_CTRL.bits() |
            EventFlags::M_ALT.bits() |
            EventFlags::M_SUPER.bits();
        const M_CTRL_SHIFT = EventFlags::M_CTRL.bits() |
            EventFlags::M_SHIFT.bits();
        const M_CTRL_ALT = EventFlags::M_CTRL.bits() |
            EventFlags::M_ALT.bits();
        const M_CTRL_SUPER = EventFlags::M_CTRL.bits() |
            EventFlags::M_SUPER.bits();
        const M_SHIFT_ALT = EventFlags::M_SHIFT.bits() |
            EventFlags::M_ALT.bits();
        const M_SHIFT_SUPER = EventFlags::M_SHIFT.bits() |
            EventFlags::M_SUPER.bits();
        const M_ALT_SUPER = EventFlags::M_ALT.bits() |
            EventFlags::M_SUPER.bits();
        const M_CTRL_SHIFT_ALT = EventFlags::M_CTRL.bits() |
            EventFlags::M_SHIFT.bits() |
            EventFlags::M_ALT.bits();
        const M_CTRL_SHIFT_SUPER = EventFlags::M_CTRL.bits() |
            EventFlags::M_SHIFT.bits() |
            EventFlags::M_SUPER.bits();
        const M_CTRL_ALT_SUPER = EventFlags::M_CTRL.bits() |
            EventFlags::M_ALT.bits() |
            EventFlags::M_SUPER.bits();
        const M_SHIFT_ALT_SUPER = EventFlags::M_SHIFT.bits() |
            EventFlags::M_ALT.bits() |
            EventFlags::M_SUPER.bits();
        const M_CTRL_SHIFT_ALT_SUPER = EventFlags::M_CTRL.bits() |
            EventFlags::M_SHIFT.bits() |
            EventFlags::M_ALT.bits()|
            EventFlags::M_SUPER.bits();
        const M_NOTHING = 0;
    }
}

impl EventFlags {
    pub(crate) fn modifiers(mut self, modifiers: KeyModifiers) -> Self {
        if (modifiers & KeyModifiers::CONTROL) != KeyModifiers::empty() {
            self.insert(EventFlags::M_CTRL);
        }
        if (modifiers & KeyModifiers::ALT) != KeyModifiers::empty() {
            self.insert(EventFlags::M_ALT);
        }
        if (modifiers & KeyModifiers::SHIFT) != KeyModifiers::empty() {
            self.insert(EventFlags::M_SHIFT);
        }
        if (modifiers & KeyModifiers::SUPER) != KeyModifiers::empty() {
            self.insert(EventFlags::M_SUPER);
        }
        self
    }
}

pub struct EventHandler<T> {
    key_handlers: Vec<fn(arg: &mut T, &mut App, &KeyEvent, EventFlags) -> bool>,
    mouse_handlers: Vec<fn(arg: &mut T, &EventHandler<T>, &mut App, &MouseEvent, EventFlags) -> bool>,
    double_click_handlers: Vec<fn(arg: &mut T, &mut App, &MouseEvent, EventFlags) -> bool>
}

impl<T> EventHandler<T> {
    pub(crate) fn new(
        key_handlers: Vec<fn(arg: &mut T, &mut App, &KeyEvent, EventFlags) -> bool>,
        mouse_handlers: Vec<fn(arg: &mut T, &EventHandler<T>, &mut App, &MouseEvent, EventFlags) -> bool>,
        double_click_handlers: Vec<fn(arg: &mut T, &mut App, &MouseEvent, EventFlags) -> bool>
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
        for key_handler in &self.key_handlers {
            if !key_handler(arg, app, &event, EventFlags::empty().modifiers(event.modifiers)) {
                break;
            }
        }
    }

    fn handle_mouse_event(&mut self, arg: &mut T, app: &mut App, event: MouseEvent) {
        for mouse_handler in &self.mouse_handlers {
            if !mouse_handler(arg, self, app, &event, EventFlags::empty().modifiers(event.modifiers)) {
                break;
            }
        }
    }

    pub(crate) fn default_double_click_handler(arg: &mut T, self_: &Self, app: &mut App, e: &MouseEvent, f: EventFlags) -> bool {
        if !matches!(e.kind, MouseEventKind::Down(_)) {
            return true;
        }
        
        let now = Instant::now();
        
        if (now - app.double_click_details.2 > DOUBLE_CLICK_DURATION) || 
            (e.column != app.double_click_details.0) ||
            (e.row != app.double_click_details.1)
        {
            app.double_click_details = (
                e.column, e.row, now
            );
        } else {
            for double_click_handler in &self_.double_click_handlers {
                if !double_click_handler(arg, app, e, f) {
                    return false;
                }
            }
        }
        true
    }
}