use crate::assets::colors::{C_TREE_FG_DIR, C_TREE_FG_FILE};
use crate::assets::constants::{TREE_FILE_LOADED_SYMBOL, TREE_FILE_MODIFIED_SYMBOL};
use crate::backend::buffers::{Buffers, BuffersLock, Inode};
use crate::backend::file_tree::NodePointer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Widget};
use std::fmt::{Debug, Formatter};
use std::fs::read_dir;
use std::path::PathBuf;

pub(crate) enum ColoringProto {
    File,
    Dir,
}

impl ColoringProto {
    pub(crate) fn draw(&self, rect: Rect, buffer: &mut ratatui::buffer::Buffer) {
        Block::new()
            .fg(match self {
                ColoringProto::File => C_TREE_FG_FILE,
                ColoringProto::Dir => C_TREE_FG_DIR,
            })
            .render(rect, buffer)
    }
}

pub(crate) struct FileTreeNode {
    path: PathBuf,
    is_dir: bool,
    expanded: bool, // Should be false for files
    dirty: bool,
    inode: Option<Inode>,
    // depth: usize,
    pub children: Vec<FileTreeNode>,
}

impl Debug for FileTreeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct FileTreeNode<'a> {
            path: &'a PathBuf,
            is_dir: bool,
            expanded: bool,
            dirty: bool,
        }
        FileTreeNode {
            path: &self.path,
            is_dir: self.is_dir,
            expanded: self.expanded,
            dirty: self.dirty,
        }
        .fmt(f)
    }
}

pub(crate) enum OnlineState {
    Nothing,
    Opened,
    Modified,
}

impl FileTreeNode {
    pub(crate) fn dir_from(path: PathBuf) -> FileTreeNode {
        Self {
            inode: None,
            path,
            dirty: true,
            is_dir: true,
            expanded: false,
            // depth,
            children: vec![],
        }
    }
    pub(crate) fn file_from(path: PathBuf) -> FileTreeNode {
        Self {
            inode: None,
            path,
            dirty: false, // Not important I think
            is_dir: false,
            expanded: false, // Not important
            // depth,
            children: vec![], // Not important
        }
    }

    pub(crate) fn expand(&mut self) {
        if !self.is_dir {
            return;
        }
        self.expanded = true;
        self.undirty();
    }

    pub(crate) fn undirty(&mut self) {
        if self.dirty {
            self.dirty = false;
            self.children.clear();
            for entry in read_dir(self.path.as_path()).unwrap() {
                // todo: remove unwrap
                let entry = entry.unwrap(); // todo: remove unwrap
                self.children.push(if entry.file_type().unwrap().is_file() {
                    // todo: remove unwrap
                    Self::file_from(entry.path())
                } else {
                    Self::dir_from(entry.path())
                })
            }
        }
    }

    pub(crate) fn handle_single_click(&mut self) {
        if self.is_dir {
            if self.expanded {
                self.expanded = false;
            } else {
                self.expand();
            }
        } else {
            BuffersLock
                .get_file_change_guard()
                .open_file_or_focus(self.path.clone());
        }
    }

    pub(crate) fn get(&self, pointer: &NodePointer) -> Option<&Self> {
        let mut parent = self;
        for &i in &pointer.inner {
            if !parent.expanded {
                return None;
            }
            parent = parent.children.get(i)?;
        }
        Some(parent)
    }
    pub(crate) fn get_mut(&mut self, pointer: &NodePointer) -> Option<&mut Self> {
        let mut parent = self;
        for &i in &pointer.inner {
            parent = parent.children.get_mut(i)?;
        }
        Some(parent)
    }

    pub(crate) fn r00t_push_string(&mut self, ptr: &NodePointer, str: &mut String) -> usize {
        let mut coloring_start_x = 0;
        str.push(' ');
        coloring_start_x += 1;
        str.push(
            match BuffersLock
                .get_check_guard()
                .get_online_state(self.get_mut(ptr).unwrap().get_inode())
            {
                // The pointer should be valid
                OnlineState::Nothing => ' ',
                OnlineState::Opened => TREE_FILE_LOADED_SYMBOL,
                OnlineState::Modified => TREE_FILE_MODIFIED_SYMBOL,
            },
        );
        coloring_start_x += 1;
        str.push(' ');
        let mut parent = self as &FileTreeNode;
        if !ptr.inner.is_empty() {
            for &i in &ptr.inner[..ptr.inner.len() - 1] {
                if i + 1 == parent.children.len() {
                    str.push_str("  ");
                } else {
                    str.push_str("│ ");
                }
                parent = parent.children.get(i).unwrap(); // The pointer should be valid
            }
            if *ptr.inner.last().unwrap() + 1 == parent.children.len() {
                // We checked (ptr.inner.len() != 0)
                str.push_str("└─");
            } else {
                str.push_str("├─");
            }
        }
        coloring_start_x += 2 * ptr.inner.len();
        let node = self.get(ptr).unwrap(); // The pointer should be valid
        str.push_str(if node.is_dir {
            if node.expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        });
        coloring_start_x += 2;
        str.push_str(node.path.file_name().unwrap().to_str().unwrap());
        str.push('\n');
        coloring_start_x
    }

    pub(crate) fn get_inode(&mut self) -> Inode {
        if let Some(inode) = self.inode {
            inode
        } else {
            let inode = Buffers::get_inode(&self.path).unwrap_or_else(|_| Inode::new_virtual());
            self.inode = Some(inode);
            inode
        }
    }

    pub(crate) fn get_coloring_proto(&self) -> ColoringProto {
        if self.is_dir {
            ColoringProto::Dir
        } else {
            ColoringProto::File
        }
    }
}
