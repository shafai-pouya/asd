use crate::assets::colors::C_LOG_TODO;
use crate::assets::constants::READ_ONLY_PATH;
use crate::backend::buffer::Buffer;
use crate::backend::file_tree_node::OnlineState;
use crate::ui::log::{LOGS, Log};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
#[cfg(test)]
use std::{
    sync::Arc,
    thread::{self, ThreadId},
};

#[cfg(not(test))]
pub static BUFFERS_LOCK: Lazy<Mutex<Buffers>> =
    Lazy::new(|| Mutex::new(unsafe { Buffers::empty() }));

#[cfg(test)]
pub static PER_THREAD_BUFFERS_LOCK: Lazy<Mutex<HashMap<ThreadId, Arc<Mutex<Buffers>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct BuffersLock;
pub static BUFFERS_DEADLOCK_PROTECTOR: AtomicBool = AtomicBool::new(false);

impl BuffersLock {
    #[cfg(not(test))]
    fn get_guard(&self) -> BuffersGuardArc {
        if BUFFERS_DEADLOCK_PROTECTOR.swap(true, Ordering::Acquire) {
            panic!("attempted to recursively lock a Mutex");
        }

        BuffersGuardArc {
            arc: BUFFERS_LOCK.deref(),
        }
    }
    #[cfg(test)]
    fn get_guard(&self) -> BuffersGuardArc {
        // We hope that deadlock won't happen
        let mut per_threads_lock = PER_THREAD_BUFFERS_LOCK.lock().unwrap();
        let tid = thread::current().id();
        let a = per_threads_lock
            .entry(tid)
            .or_insert_with(|| Arc::new(Mutex::new(unsafe { Buffers::empty() })));
        BuffersGuardArc { arc: a.clone() }
    }
    pub fn get_render_guard(&self) -> BuffersRenderGuardArc {
        BuffersRenderGuardArc {
            inner: self.get_guard(),
        }
    }
    pub fn get_change_guard(&self) -> BuffersChangeGuardArc {
        BuffersChangeGuardArc {
            inner: self.get_guard(),
        }
    }

    pub fn get_check_guard(&self) -> BuffersCheckGuardArc {
        BuffersCheckGuardArc {
            inner: self.get_guard(),
        }
    }

    pub fn get_file_change_guard(&self) -> BuffersFileChangeGuardArc {
        BuffersFileChangeGuardArc {
            inner: self.get_guard(),
        }
    }

    #[inline]
    pub fn handle_checkpoint_timers(&self) {
        let now = Instant::now();

        let buffers = self.get_guard();
        let mut buffers = buffers.lock();

        for buffer in buffers.buffers.inner.values_mut() {
            if let Some(t) = buffer.checkpoints.little_timer_deadline
                && now >= t
            {
                buffer.checkpoints.little_timer_deadline = None;
                buffer.commit();
            }

            if let Some(t) = buffer.checkpoints.big_timer_deadline
                && now >= t
            {
                buffer.checkpoints.big_timer_deadline = None;
                buffer.commit();
            }
        }
    }

    pub(crate) fn quit_current(&self) {
        let buffers = self.get_file_change_guard();
        let mut buffers = buffers.lock();
        if buffers.buffers.buffers.inner.len() == 1 {
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

    pub(crate) fn force_quit_current(&self) {
        let buffers = BuffersLock.get_file_change_guard();
        let mut buffers = buffers.lock();
        if buffers.buffers.buffers.inner.len() == 1 {
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
}

struct BuffersGuardArc {
    #[cfg(test)]
    arc: Arc<Mutex<Buffers>>,
    #[cfg(not(test))]
    arc: &'static Mutex<Buffers>,
}

impl BuffersGuardArc {
    pub fn lock(&self) -> BuffersGuard<'_> {
        BuffersGuard {
            buffers: self.arc.lock().unwrap(),
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

pub struct BuffersChangeGuardArc {
    inner: BuffersGuardArc,
}
impl BuffersChangeGuardArc {
    pub fn lock(&self) -> BuffersChangeGuard<'_> {
        BuffersChangeGuard {
            buffers: self.inner.lock(),
        }
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

pub struct BuffersRenderGuardArc {
    inner: BuffersGuardArc,
}
impl BuffersRenderGuardArc {
    pub fn lock(&self) -> BuffersRenderGuard<'_> {
        BuffersRenderGuard {
            buffers: self.inner.lock(),
        }
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
pub struct BuffersCheckGuardArc {
    inner: BuffersGuardArc,
}
impl BuffersCheckGuardArc {
    pub fn lock(&self) -> BuffersCheckGuard<'_> {
        BuffersCheckGuard {
            buffers: self.inner.lock(),
        }
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
        for buffer in self.buffers.buffers.inner.values_mut() {
            buffer.commit();
        }
    }

    pub(crate) fn any_modified(&self) -> bool {
        self.buffers.buffers.inner.iter().any(|(_, a)| a.modified)
    }
}

pub struct BuffersFileChangeGuardArc {
    inner: BuffersGuardArc,
}
impl BuffersFileChangeGuardArc {
    pub fn lock(&self) -> BuffersFileChangeGuard<'_> {
        BuffersFileChangeGuard {
            buffers: self.inner.lock(),
        }
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
            Buffer::new_utf8(
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
    pub fn open_custom(&mut self, showing_name: String, content: &str) {
        let inode = Inode::new_virtual();
        self.buffers.buffers.active_inode = inode;
        self.buffers.buffers.inner.insert(
            inode,
            Buffer::new_utf8(PathBuf::from(READ_ONLY_PATH), showing_name, content),
        );
    }
    pub(crate) fn remove_self(&mut self) {
        let buffers: &mut Buffers = &mut self.buffers.buffers;
        let buffer = buffers.active_mut();
        if buffer.try_quit().is_ok() {
            buffers.inner.remove(&buffers.active_inode);
            buffers.active_inode = *buffers.inner.iter().next().unwrap().0; // todo: remove unwrap
        }
    }
    pub(crate) fn force_remove_self(&mut self) {
        let buffers: &mut Buffers = &mut self.buffers.buffers;
        buffers.inner.remove(&buffers.active_inode);
        buffers.active_inode = *buffers.inner.iter().next().unwrap().0; // todo: remove unwrap
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
    active_inode: Inode,
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
}
