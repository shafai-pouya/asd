use crate::App;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::widgets::Paragraph;

pub(crate) fn render_tree(app: &mut App, tree_area: Rect, buf: &mut Buffer) {
    let Some(file_tree) = &mut app.file_tree else { return; };
    Paragraph::new(file_tree.get_all_texts_from_len(file_tree.scrollbar_y, app.last_tree_rect.height, &app.buffers, buf, &mut app.virtual_inode_counter))
        .render(tree_area, buf)
}