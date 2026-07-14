use crate::backend::buffers::Buffers;
use crate::backend::file_tree_node::FileTreeNode;
use crate::ui::log::Log;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::path::PathBuf;

pub const FILE_MODIFIED: char = '●';
pub const FILE_LOADED: char = '○';

#[derive(Debug)]
pub(crate) struct NodePointer {
    pub inner: Vec<usize>,
}

impl NodePointer {
    pub(crate) fn get_on_nth_line(&mut self, mut n: usize, tree: &FileTree) -> Result<(), usize> {
        let start_n = n;
        loop {
            if n == 0 {
                return Ok(())
            } else {
                match self.push_0(&tree.root) {
                    Ok(()) => {}
                    Err(()) => return Err(start_n - n),
                };
                n -= 1;
            }
        }
    }
}

impl NodePointer {
    pub(crate) fn new() -> Self {
        Self { inner: vec![] }
    }

    #[inline]
    pub(crate) fn next(&mut self, root: &FileTree) -> Result<(), ()> {
        self.push_0(&root.root)
    }

    pub(crate) fn push_0(&mut self, root: &FileTreeNode) -> Result<(), ()> {
        self.inner.push(0);
        while root.get(self).is_none() {
            self.inner.truncate(self.inner.len() - 1);
            let Some(last) = self.inner.last_mut() else { return Err(()); };
            *last += 1;
        }
        Ok(())
    }
}

pub(crate) struct FileTree {
    pub width: u16,
    pub scrollbar_y: usize,
    pub root: FileTreeNode,
}

impl FileTree {
    pub(crate) fn new(path: PathBuf) -> Self {
        let mut self_ = Self {
            width: 20,
            scrollbar_y: 0,
            root: FileTreeNode::dir_from(path),
        };
        self_.root.expand();
        self_
    }
    pub(crate) fn handle_event(&mut self, event: MouseEvent, buffers: &mut Buffers, logs: &mut Vec<Log>, last_tree_rect: Rect) {
        // let MouseEventKind::Down(MouseButton::Left) = event.kind else { return; };
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {}
            MouseEventKind::ScrollDown => {
                self.scrollbar_y += 5;
                let mut ptr = NodePointer::new();
                let lines_n = ptr.get_on_nth_line(usize::MAX, &self).unwrap_err();
                // let lines_n = self.root.get_on_nth_line(usize::MAX, &mut ptr).unwrap_err();
                if lines_n + 10 < self.scrollbar_y + last_tree_rect.height as usize {
                    self.scrollbar_y = (lines_n + 10).saturating_sub(last_tree_rect.height as usize);
                }
                return;
            }
            MouseEventKind::ScrollUp => {
                self.scrollbar_y = self.scrollbar_y.saturating_sub(5);
                return;
            }
            _ => {
                return;
            }
        }

        let mut pointer = NodePointer::new();
        let n = event.row as usize + self.scrollbar_y;
        if pointer.get_on_nth_line(n, &self).is_err() { // todo: scrollbar
            return;
        }

        let a = self.root.get_mut(&pointer).unwrap();
        a.handle_single_click(buffers, logs);
        // todo
    }
    pub(crate) fn handle_separator_event(&mut self, event: MouseEvent, next_is_separator: &mut bool) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                *next_is_separator = true;
            }
            MouseEventKind::Drag(MouseButton::Left) => {}
            MouseEventKind::Up(MouseButton::Left) => {
                *next_is_separator = false;
                return;
            },
            _ => {
                return;
            }
        }

        self.width = event.column;
    }
    pub(crate) fn get_all_texts_from_len(&mut self, start: usize, len: u16, buffers: &Buffers, buffer: &mut ratatui::buffer::Buffer) -> String {
        let mut ptr = NodePointer::new();
        let mut to_return = String::new();
        if ptr.get_on_nth_line(start, &self).is_err() {
            return to_return;
        }
        for displaying_row in 0..len {
            let coloring_x = self.root.r00t_push_string(&ptr, &mut to_return, buffers) as u16;
            let coloring_protocol = self.root.get(&ptr).unwrap().get_coloring_proto();
            coloring_protocol.draw(Rect {
                x: coloring_x,
                y: displaying_row,
                width: self.width - coloring_x,
                height: 1,
            }, buffer);
            if ptr.next(&self).is_err() {
                return to_return;
            };
        }
        to_return
    }
}