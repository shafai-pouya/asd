mod assets;
mod ui;
mod backend;
mod edit_operators;
mod edit_controller;

use crate::assets::colors::colors::{C_ERROR, C_HINT};
use crate::backend::buffer::Buffer;
use crate::backend::buffers::{Buffers, Inode};
use crate::backend::file_tree::FileTree;
use crate::backend::modes::editor_mode::EditorMode;
use crate::backend::modes::Mode;
use crate::ui::base::render_base;
use crate::ui::cursor::TerminalCursor;
use crate::ui::log::Log;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::layout::Position;
use ratatui::{layout::Rect, DefaultTerminal, Frame};
use std::collections::HashMap;
use std::env::args;
#[cfg(debug_assertions)]
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(debug_assertions)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct App {
    /// If actives, The application exits in the next loop
    exit: bool,

    buffers: Buffers,


    file_tree: Option<FileTree>,
    // buffer: Buffer,

    pub logs: Vec<Log>,

    /// Used for setting the real terminal cursor
    terminal_cursor: TerminalCursor,
    /// Last frame content rect, shows the area for **content** (Line numbers doesn't count) when
    /// there are no logs showing
    last_content_rect: Rect,
    /// Last frame content rect, shows the area for file tree when
    /// there are no logs showing
    last_tree_rect: Rect,
    /// Last frame separator rect
    last_tree_and_content_separator_rect: Rect,
    /// Used by [`EventHandler`] to detect double clicks
    double_click_details: (u16, u16, Instant),
    /// If actives, The mode will change for the next loop
    change_mode: Option<Box<dyn Mode>>,
    next_is_separator_event: bool,


    pub virtual_inode_counter: usize
}

pub const POLL_DURATION: Duration = Duration::from_millis(100);

impl App {
    #[inline]
    fn no_file() -> Self {
        // todo: change this function
        // todo!();
        let _ = execute!(
            std::io::stdout(),
            DisableMouseCapture,
        );
        eprintln!("Use the following syntax:\n $ asd /path/to/file/or/dir");
        std::process::exit(1);
        // Self::file("src/main.rs")
    }

    #[inline]
    fn file(path: &str) -> Self {
        let path = Path::new(path);
        let mut logs = vec![];
        let mut vic = 0;
        let mut hm = HashMap::new();
        let inode = Buffers::get_inode(&path).unwrap_or_else(|e| {
            logs.push(Log {
                message: format!("Failed to get Inode: {e}"),
                color: C_ERROR,
            });
            Inode::virtual_generator(&mut vic)
        });
        hm.insert(inode, Buffer::new(PathBuf::from(path), &mut logs));
        let is_dir = match path.metadata() {
            Ok(meta) => meta.file_type().is_dir(),
            Err(_) => false,
        }; // Safety: We just got metadata in a few lines ago
        let file_tree = if is_dir {
            Some(FileTree::new(PathBuf::from(path)))
        } else {
            None
        };
        Self {
            buffers: Buffers::new(
                hm,
                inode
            ),
            virtual_inode_counter: vic,
            exit: false,
            next_is_separator_event: false,
            file_tree,
            terminal_cursor: TerminalCursor::new(),
            last_content_rect: Rect::default(),
            last_tree_rect: Rect::default(),
            last_tree_and_content_separator_rect: Rect::default(),
            double_click_details: (u16::MAX, u16::MAX, Instant::now()),
            change_mode: None,
            logs,
        }
    }
}

impl App {
    #[inline]
    pub(crate) fn run(mut self, terminal: &mut DefaultTerminal, mode: &mut Box<dyn Mode>) {
        while !self.exit {
            self.draw(terminal, mode);
            self.handle_events(mode);
            self.handle_checkpoint_timers();
        }
    }

    #[inline]
    pub(crate) fn draw(&mut self, terminal: &mut DefaultTerminal, mode: &mut Box<dyn Mode>) {
        terminal.draw(|frame| {
            self.render(frame, mode);
        }).unwrap();  // I can't do anything. I let it crash
        self.terminal_cursor.render2(terminal);
    }

    #[inline]
    fn handle_events(&mut self, mode: &mut Box<dyn Mode>) {
        if event::poll(POLL_DURATION).unwrap() {  // I can't do anything. I let it crash
            self.handle_event(mode, event::read().unwrap()); // I can't do anything. I let it crash
        }
    }

    #[inline]
    fn handle_event(&mut self, mode: &mut Box<dyn Mode>, event: Event) {
        match event {
            Event::Mouse(me)
            if self.next_is_separator_event ||
                self.last_tree_and_content_separator_rect.contains(Position::new(me.column, me.row)) => {
                self.file_tree.as_mut().unwrap().handle_separator_event(
                    me, &mut self.next_is_separator_event
                ) // Safety: last_tree_rect should be empty when file tree isn't present
            }
            Event::Mouse(me) if self.last_tree_rect.contains(Position::new(me.column, me.row)) => {
                self.file_tree.as_mut().unwrap().handle_event(
                    me, &mut self.buffers, &mut self.logs, self.last_tree_rect, &mut self.virtual_inode_counter
                ) // Safety: last_tree_rect should be empty when file tree isn't present
            }
            _ => {
                mode.handle_event(self, event);
                if let Some(m) = self.change_mode.take() {
                    *mode = m;
                }
            }
        }
    }

    #[inline]
    fn render(&mut self, frame: &mut Frame, mode: &mut Box<dyn Mode>) {
        render_base(self, frame, !mode.needs_terminal_cursor());
        self.terminal_cursor.render1(frame);
        mode.render_function(frame);
    }

    #[inline]
    fn handle_checkpoint_timers(&mut self) {
        let now = Instant::now();

        for (_, buffer) in unsafe { self.buffers.inner_mut() } {
            if let Some(t) = buffer.checkpoints.little_timer_deadline {
                if now >= t {
                    buffer.checkpoints.little_timer_deadline = None;
                    buffer.commit(&mut self.logs);
                }
            }

            if let Some(t) = buffer.checkpoints.big_timer_deadline {
                if now >= t {
                    buffer.checkpoints.big_timer_deadline = None;
                    buffer.commit(&mut self.logs);
                }
            }
        }
    }


    fn op_set_clipboard(data: &[u8], logs: &mut Vec<Log>) {
        match Command::new("xclip")
            .args([
                "-selection",
                "clipboard",
            ])
            .stdin(Stdio::piped())
            .spawn() {
            Ok(mut clip_command) => {
                let mut stdin = clip_command.stdin.take().unwrap(); // Safety: stdin is piped
                match stdin.write_all(data) {
                    Ok(_) => {}
                    Err(e) => {
                        logs.push(Log {
                            message: format!("[E:{}] Error opening stdin of xclip to copy data: {}", e.kind() as u32, e.kind().to_string()),
                            color: C_ERROR,
                        });
                        return;
                    }
                };
                drop(stdin);
                match clip_command.wait() {
                    Ok(_) => {}
                    Err(e) => {
                        logs.push(Log {
                            message: format!("[E:{}] Error waiting for xclip to copy data: {}", e.kind() as u32, e.kind().to_string()),
                            color: C_ERROR,
                        });
                    }
                };
            }
            Err(e) => {
                logs.push(Log {
                    message: format!("[E:{}] Error opening xclip to copy data: {}", e.kind() as u32, e.kind().to_string()),
                    color: C_ERROR,
                });
            }
        }
    }

    fn op_get_clipboard(logs: &mut Vec<Log>) -> Option<String> {
        let text = String::from_utf8(
            match Command::new("xclip")
                .args([
                    "-selection",
                    "clipboard",
                    "-o"
                ])
                .output()
            {
                Ok(o) => o,
                Err(e) => {
                    logs.push(Log {
                        message: format!("[E:{}] Error getting the clipboard: {}", e.kind() as u32, e.kind().to_string()),
                        color: C_ERROR,
                    });
                    return None;
                }
            }
                .stdout
        )
            .unwrap();
        Some(text)
    }


    fn operate_quit(&mut self, ) {
        self.buffers.commit_all(&mut self.logs);
        if !self.buffers.any_modified() {
            self.exit = true;
            return;
        }
        self.logs.push(Log {
            message: "[E:i1] Cannot exit: modified buffer exists".to_string(),
            color: C_ERROR,
        });
        self.logs.push(Log {
            message: "[HINT] Use ctrl+alt+q if you sure you want to exit".to_string(),
            color: C_HINT
        });
    }
    fn operate_force_quit(&mut self) {
        self.exit = true;
    }
}

fn main() {

    // The code in the following unsafe block only wrote because of some editor problems when
    // rendering tui and handling events. So, it opens the editor on another terminal
    #[cfg(debug_assertions)]
    unsafe {
        let file = OpenOptions::new()
           .read(true)
           .write(true)
           .open("/dev/pts/1")
            .unwrap();


        libc::dup2(file.as_raw_fd(), 0); // stdin
        libc::dup2(file.as_raw_fd(), 1); // stdout
        libc::dup2(file.as_raw_fd(), 2); // stderr

        libc::setsid();
        libc::ioctl(file.as_raw_fd(), libc::TIOCSCTTY, 0);

        // let (w, h) = crossterm::terminal::size()?;
        // print!(
        //     "\x1bP@kitty-cmd{}\
        //  \x1b\\",
        //     format!(
        //         r#"{{"cmd":"resize-os-window","version":[0,43,0],"payload":{{"width":{},"height":{},"unit":"cells"}}}}"#,
        //         w+4,
        //         h+4
        //     )
        // );

        std::io::stdout().flush().unwrap();
    }


    // Handle args
    let mut args = args().into_iter();
    args.next();
    let app = if let Some(arg) = args.next() {
        App::file(&arg)
    } else {
        App::no_file()
    };
    assert_eq!(args.next(), None);


    // error handling
    color_eyre::install().unwrap(); // You can panic here
    let old_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = execute!(
            std::io::stdout(),
            DisableMouseCapture,
        );

        old_hook(panic_info);
    }));

    // init
    let mut terminal = ratatui::init();
    enable_raw_mode().unwrap(); // you can panic here
    execute!(
        std::io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        SetCursorStyle::BlinkingUnderScore,
    ).unwrap(); // Usually ok, but you can panic here

    // app
    let mut mode = Box::new(EditorMode::new()) as Box<dyn Mode>;
    app.run(&mut terminal, &mut mode);

    // end
    execute!(
        std::io::stdout(),
        DisableMouseCapture,
    ).unwrap(); // Usually ok, but you can panic here;
    ratatui::restore();
}
