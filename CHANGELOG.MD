# Changelog

## [1.0.2]

### Added

- Add a `debug-tools` Cargo feature for development and debugging.

### Infrastructure

- Add the initial text styling infrastructure for future syntax highlighting and other text styling features. Styled text rendering is not yet implemented, and this infrastructure has not been tested.

### Fixed

- Fix a crash caused by unintended cursor merging during multicursor selection.
- Fix crashes in `op_materialize_virtual_spaces` when materializing selections containing both virtual spaces and real characters.

### Changed

- Refactor `LOGS`, `BUFFERS`, and `VIRTUAL_INODE_COUNTER` into static variables.
- Refactor various parts of the codebase.
- Add tests across the codebase.


## [1.0.1]

Bug Fix

### Bug fixes:

- Terminal cursor wasn't showing in the startup.

### Changes:

- Moved the main code to `asd` directory for future compatibilities.

## [1.0.0]

Initial release

### Added

- Multi-buffer
- File tree (experimental, unstable)
- Mouse support
- UTF-8 support
- Raw mode support
- Undo / Redo
- Save As
- Built-in interactive help guide
- Multi-cursor
- Menu (Still under development)
- Clock in the status bar
- Scrolling
- Emoji showing (buggy, but usually works well)