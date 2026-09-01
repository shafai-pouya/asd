use crate::App;
use crate::assets::colors::colors::C_LOG_TODO;
use crate::assets::constants::READ_ONLY_PATH;
use crate::backend::buffer::Buffer;
use crate::backend::file_tree_node::OnlineState;
use crate::ui::log::{LOGS, Log};
use crossterm::event::KeyEvent;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

pub static BUFFERS_LOCK: Lazy<Mutex<Buffers>> =
    Lazy::new(|| Mutex::new(unsafe { Buffers::empty() }));

pub struct BuffersType;
pub static BUFFERS: BuffersType = BuffersType;
pub static BUFFERS_DEADLOCK_PROTECTOR: AtomicBool = AtomicBool::new(false);

impl BuffersType {
    fn get_guard(&self) -> BuffersGuard<'_> {
        if BUFFERS_DEADLOCK_PROTECTOR.swap(true, Ordering::Acquire) {
            panic!("attempted to recursively lock a Mutex");
        }

        BuffersGuard {
            buffers: BUFFERS_LOCK.lock().unwrap(),
        }
    }
    pub fn get_render_guard(&self) -> BuffersRenderGuard<'_> {
        BuffersRenderGuard {
            buffers: self.get_guard(),
        }
    }
    pub fn get_change_guard(&self) -> BuffersChangeGuard<'_> {
        BuffersChangeGuard {
            buffers: self.get_guard(),
        }
    }

    pub fn get_check_guard(&self) -> BuffersCheckGuard<'_> {
        BuffersCheckGuard {
            buffers: self.get_guard(),
        }
    }

    pub fn get_file_change_guard(&self) -> BuffersFileChangeGuard<'_> {
        BuffersFileChangeGuard {
            buffers: self.get_guard(),
        }
    }

    #[inline]
    pub fn handle_checkpoint_timers(&self) {
        let now = Instant::now();

        let mut buffers = self.get_guard();

        for (_, buffer) in unsafe { buffers.buffers.inner_mut() } {
            if let Some(t) = buffer.checkpoints.little_timer_deadline {
                if now >= t {
                    buffer.checkpoints.little_timer_deadline = None;
                    buffer.commit();
                }
            }

            if let Some(t) = buffer.checkpoints.big_timer_deadline {
                if now >= t {
                    buffer.checkpoints.big_timer_deadline = None;
                    buffer.commit();
                }
            }
        }
    }
}

struct BuffersGuard<'a> {
    buffers: MutexGuard<'a, Buffers>,
}

impl Drop for BuffersGuard<'_> {
    fn drop(&mut self) {
        BUFFERS_DEADLOCK_PROTECTOR.store(false, Ordering::Release);
    }
}

pub struct BuffersChangeGuard<'a> {
    buffers: BuffersGuard<'a>,
}

impl<'a> BuffersChangeGuard<'a> {
    #[allow(dead_code)]
    pub fn inner(&self) -> &Buffers {
        self.buffers.buffers.deref()
    }
    #[allow(dead_code)]
    pub fn inner_mut(&mut self) -> &mut Buffers {
        self.buffers.buffers.deref_mut()
    }
}

pub struct BuffersRenderGuard<'a> {
    buffers: BuffersGuard<'a>,
}
impl<'a> BuffersRenderGuard<'a> {
    pub fn inner(&self) -> &Buffers {
        self.buffers.buffers.deref()
    }
    pub fn inner_mut(&mut self) -> &mut Buffers {
        self.buffers.buffers.deref_mut()
    }
}
pub struct BuffersCheckGuard<'a> {
    buffers: BuffersGuard<'a>,
}
impl BuffersCheckGuard<'_> {
    pub fn get_online_state(&self, inode: Inode) -> OnlineState {
        match self.buffers.buffers.inner.get(&inode) {
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

    pub(crate) fn commit_all(&mut self) {
        for (_, buffer) in &mut self.buffers.buffers.inner {
            buffer.commit();
        }
    }

    pub(crate) fn any_modified(&self) -> bool {
        self.buffers.buffers.inner.iter().any(|(_, a)| a.modified)
    }
}

pub struct BuffersFileChangeGuard<'a> {
    buffers: BuffersGuard<'a>,
}
impl BuffersFileChangeGuard<'_> {
    pub(crate) fn open_help(&mut self) {
        let inode = Inode::new_virtual();
        self.buffers.buffers.inner.insert(
            inode,
            Buffer::new_custom(
                PathBuf::from(READ_ONLY_PATH),
                "help.txt (READONLY)".to_string(),
                include_str!("../assets/help.txt"),
            ),
        );
        self.buffers.buffers.active_inode = inode;
    }

    pub(crate) fn insert(&mut self, index: Inode, buffer: Buffer) {
        self.buffers.buffers.inner.insert(index, buffer);
    }

    pub(crate) fn open_file_or_focus(&mut self, path: PathBuf) {
        let inode = Buffers::get_inode(&path).unwrap_or_else(|_| Inode::new_virtual());
        self.buffers.buffers.active_inode = inode;
        if !self.buffers.buffers.inner.contains_key(&inode) {
            self.insert(inode, Buffer::new_from_file(path));
        }
    }
}

static VIRTUAL_INODE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Inode {
    Real(u64),
    Virtual(usize), // Not yet
}

impl Inode {
    pub(crate) fn new_virtual() -> Inode {
        Self::Virtual(VIRTUAL_INODE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub(crate) struct Buffers {
    inner: HashMap<Inode, Buffer>,
    pub active_inode: Inode,
}

impl Buffers {
    pub unsafe fn empty() -> Self {
        Self {
            inner: HashMap::new(),
            active_inode: Inode::Virtual(0),
        }
    }

    pub(crate) fn active(&self) -> &Buffer {
        &self.inner[&self.active_inode]
    }
    pub(crate) fn active_mut(&mut self) -> &mut Buffer {
        self.inner.get_mut(&self.active_inode).unwrap() // Safety: active_node should be valid always
    }

    pub(crate) fn get_inode(path: &Path) -> Result<Inode, std::io::Error> {
        Ok(Inode::Real(path.metadata()?.ino()))
    }

    #[inline]
    pub(crate) unsafe fn inner_mut(&mut self) -> &mut HashMap<Inode, Buffer> {
        &mut self.inner
    }

    pub(crate) fn quit_current_evt(_: &mut App, _: &KeyEvent) {
        let mut buffers = BUFFERS_LOCK.lock().unwrap();
        if buffers.inner.len() == 1 {
            // todo!()
            LOGS.push(Log {
                message: "This buffer is the only opened buffer. try opening another one and close this one (todo)".to_string(),
                color: C_LOG_TODO,
                handler: None,
            });
            return;
        }
        buffers.remove_self();
    }

    pub(crate) fn force_quit_current_evt(_: &mut App, _: &KeyEvent) {
        let mut buffers = BUFFERS_LOCK.lock().unwrap();
        if buffers.inner.len() == 1 {
            // todo!()
            LOGS.push(Log {
                message: "This buffer is the only opened buffer. try opening another one and close this one (todo)".to_string(),
                color: C_LOG_TODO,
                handler: None,
            });
            return;
        }
        buffers.force_remove_self();
    }

    pub(crate) fn open_help_evt(_: &mut App, _: &KeyEvent) {
        let mut buffers = BUFFERS.get_file_change_guard();
        buffers.open_help()
    }
}

impl Buffers {
    pub(crate) fn remove_self(&mut self) {
        let buffer = self.active_mut();
        if buffer.try_quit().is_ok() {
            self.inner.remove(&self.active_inode);
            self.active_inode = *self.inner.iter().next().unwrap().0; // todo: remove unwrap
        }
    }
    pub(crate) fn force_remove_self(&mut self) {
        self.inner.remove(&self.active_inode);
        self.active_inode = *self.inner.iter().next().unwrap().0; // todo: remove unwrap
    }
}
