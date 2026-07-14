use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use crossterm::event::KeyEvent;
use crate::App;
use crate::assets::colors::colors::C_TODO;
use crate::backend::buffer::Buffer;
use crate::backend::event_handler::EventFlags;
use crate::backend::file_tree_node::OnlineState;
use crate::ui::log::Log;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Inode {
    Real(u64),
    Virtual(usize), // Not yet
}

impl Inode {
    pub(crate) fn virtual_generator(virtual_inode_counter: &mut usize) -> Inode {
        *virtual_inode_counter += 1;
        Self::Virtual(*virtual_inode_counter)
    }
}

pub(crate) struct Buffers {
    inner: HashMap<Inode, Buffer>,
    pub active_inode: Inode,
}

impl Buffers {
    pub(crate) fn new(inner: HashMap<Inode, Buffer>, active_inode: Inode) -> Buffers {
        Self { inner, active_inode}
    }

    pub(crate) fn commit_all(&mut self, logs: &mut Vec<Log>) {
        for (_, buffer) in &mut self.inner {
            buffer.commit(logs);
        }
    }

    pub(crate) fn any_modified(&self) -> bool {
        self.inner.iter()
            .any(|(_, a)| a.modified)
    }

    pub(crate) fn active(&self) -> &Buffer {
        &self.inner[&self.active_inode]
    }
    pub(crate) fn active_mut(&mut self) -> &mut Buffer {
        self.inner.get_mut(&self.active_inode).unwrap() // Safety: active_node should be valid always
    }

    pub(crate) fn open_file_or_focus(&mut self, path: PathBuf, logs: &mut Vec<Log>, virtual_inode_counter: &mut usize) {
        let inode = Self::get_inode(&path).unwrap_or_else(|_| {
            Inode::virtual_generator(virtual_inode_counter)
        });
        self.active_inode = inode;
        if !self.inner.contains_key(&inode) {
            self.insert(inode, Buffer::new(path, logs));
        }
    }
    
    pub(crate) fn get_inode(path: &Path) -> Result<Inode, std::io::Error> {
        Ok(Inode::Real(path.metadata()?.ino()))
    }

    
    pub(crate) fn get_online_state(&self, index: Inode) -> OnlineState {
        match self.inner.get(&index) {
            None => OnlineState::Nothing,
            Some(buffer) => {
                if buffer.modified {
                    OnlineState::Modified
                } else {
                    OnlineState::Opened
                }
            }
        }
    }
    
    #[inline]
    pub(crate) unsafe fn inner_mut(&mut self) -> &mut HashMap<Inode, Buffer> {
        &mut self.inner
    }

    pub(crate) fn quit_current_evt(app: &mut App, _: &KeyEvent, _: EventFlags) {
        if app.buffers.inner.len() == 1 {
            // todo!()
            app.logs.push(Log {
                message: "This buffer is the only opened buffer. try opening another one and close this one (todo)".to_string(),
                color: C_TODO,
            });
            return
        }
        app.buffers.remove_self(&mut app.logs);
    }
}


impl Buffers {
    pub(crate) fn remove_self(&mut self, logs: &mut Vec<Log>) {
        let buffer = self.active_mut();
        if buffer.try_quit(logs).is_ok() {
            self.inner.remove(&self.active_inode);
            self.active_inode = *self.inner.iter().next().unwrap().0; // todo: remove unwrap
        }
    }

    pub(crate) fn insert(&mut self, index: Inode, buffer: Buffer) {
        self.inner.insert(index, buffer);
    }
}