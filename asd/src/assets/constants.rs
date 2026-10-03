use std::time::Duration;

/// The max duration which the code waits for events. It you set it to a lower value, it uses more cpu usage, and if
/// you set it to a high value, frames will draw more lately if you don't receive any events before this time
pub const POLL_DURATION: Duration = Duration::from_millis(100);

/// This file will be bound to read-only buffers.
pub const READ_ONLY_PATH: &str = "/dev/full";

/// The max number of stored checkpoints. If checkpoints become more than this number, [`N_DRAIN_CHECKPOINTS`] of them
/// will drain from the beginning
pub(crate) const N_MAX_CHECKPOINTS: usize = 1000;
/// See [`N_MAX_CHECKPOINTS`]
pub(crate) const N_DRAIN_CHECKPOINTS: usize = 100;

/// The big timer duration. See [`crate::backend::checkpoint::checkpoints::Checkpoints::big_timer_deadline`]
pub(crate) const DURATION_BIG_TIMER: Duration = Duration::from_secs(2);
/// The small timer duration. See [`crate::backend::checkpoint::checkpoints::Checkpoints::small_timer_deadline`]
pub(crate) const DURATION_SMALL_TIMER: Duration = Duration::from_millis(500);

/// The max time the user can wait between his/her first and second click to be counted as a double click
pub const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(300);

/// The max number of [`crate::backend::display_char::DisplayChar`]s for a [`crate::backend::little_string::LittleString`] to be stored in stack
pub const N_MAX_LITTLE: usize = 10;

/// The symbol which will be used in the file tree for all files which are modified
pub const TREE_FILE_MODIFIED_SYMBOL: char = '●';
/// The symbol which will be used in the file tree for all files which are opened
pub const TREE_FILE_LOADED_SYMBOL: char = '○';

/// The max empty cols for the scrollbar used as padding.
pub const SCROLLBAR_EMPTY_COLS_PADDING: usize = 20;
/// The max empty rows for the scrollbar used as padding.
pub const SCROLLBAR_EMPTY_ROWS_PADDING: usize = 10;
