use crate::ui::bar::render_bar;
use crate::ui::file::render_file;
use crate::ui::log::{get_logs_height, render_logs};
use crate::ui::render_tree::render_tree;
use crate::App;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Widget};
use ratatui::Frame;
use crate::assets::colors::colors::{C_BG_BAR, C_BG_NORMAL};

pub(crate) fn render_base(app: &mut App, frame: &mut Frame, can_use_cursor: bool) {
    let rows = Layout::vertical([
        Constraint::default(), Constraint::Length(1), Constraint::Length(get_logs_height(app)), Constraint::Length(1)
    ]);
    let [base_area, bar, logs_area, under_bar] = frame.area().layout(&rows);

    let [tree_area, separator_area, file_scroll_area] = if let Some(tree) = &app.file_tree {
        let layout = Layout::horizontal([
            Constraint::Length(tree.width), Constraint::Length(1), Constraint::Min(1)
        ]);
        base_area.layout(&layout)
    } else {
        [Rect::new(0, 0, 0, 0), Rect::new(0, 0, 0, 0), base_area]
    };

    let layout = Layout::vertical([Constraint::default(), Constraint::Length(1)]);
    let [file_area, scroll_area] = file_scroll_area.layout(&layout);

    Block::new()
        .bg(C_BG_NORMAL)
        .render(base_area, frame.buffer_mut());
    Block::new()
        .bg(C_BG_BAR)
        .render(separator_area, frame.buffer_mut());

    Block::new()
        .bg(C_BG_NORMAL)
        .render(scroll_area, frame.buffer_mut());

    Block::new()
        .bg(C_BG_NORMAL)
        .render(logs_area, frame.buffer_mut());


    render_bar(app, bar, under_bar, frame.buffer_mut());

    let last_content_rect = Rect {
        height: frame.area().height - 3,
        ..render_file(app, file_area, scroll_area, frame.buffer_mut(), can_use_cursor)
    };
    app.last_content_rect = last_content_rect;

    render_tree(app, tree_area, frame.buffer_mut());

    app.last_tree_rect = Rect {
        height: frame.area().height - 2,
        ..tree_area
    };
    
    app.last_tree_and_content_separator_rect = Rect {
        height: frame.area().height - 2,
        ..separator_area
    };

    render_logs(app, frame, logs_area);
}
