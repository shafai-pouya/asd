# ASD
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Unix-blue)
![Rust](https://img.shields.io/badge/Rust-1.90.0-orange)
![Status](https://img.shields.io/badge/status-active-success)

[//]: # (![GitHub Release]&#40;https://img.shields.io/github/v/release/shafai-pouya/asd&#41;)
[//]: # (![GitHub Downloads &#40;all assets, all releases&#41;]&#40;https://img.shields.io/github/downloads/shafai-pouya/asd/total&#41;)

A fast terminal-based text editor written in Rust.

## Features

- Fast
- Supports UTF-8 and Raw byte files
- Multi-buffer
- Multi-cursor
- File tree (unstable yet)
- Undo / Redo
- Mouse Support

![alt](docs/Base%20gif.gif)


## Installation

> [!Note]
> ASD currently supports Unix-like systems only.

[//]: # (### Binary release)
[//]: # (You can download the pre-built binaries for linux from the [release page]&#40;https://github.com/shafai-pouya/asd/releases&#41;)
[//]: # ()
[//]: # (### Or you can install it manually)
#### Step 1:
Install rust. Installation instructions are available at: https://rust-lang.org/tools/install/

#### Step 2:
Clone the repo
```shell
git clone https://github.com/shafai-pouya/asd.git
cd asd
```

#### Step 3:
Run the installation file as root
```shell
sudo ./install.sh
```

## Usage:

| Command                  | Usage                              | Notes    |
|--------------------------|------------------------------------|----------|
| `asd /path/to/file`      | open a file                        |          |
| `asd /path/to/directory` | open the file tree for a directory | unstable |
| `asd --help`             | open the asd guide                 |          |


## Key bindings:

| Key bind     | Usage                                                 |
|--------------|-------------------------------------------------------|
| Ctrl+S       | Save the file                                         |
| Ctrl+Alt+S   | Save the file as                                      |
| Ctrl+Q       | Quit                                                  |
| Ctrl+Alt+Q   | Quit even if there are unsaved buffers                |
| Ctrl+C       | Clipboard Copy (Internal Clipboard)                   |
| Ctrl+X       | Clipboard Cut (Internal Clipboard)                    |
| Ctrl+V       | Clipboard Paste (Internal Clipboard)                  |
| Ctrl+Z       | Undo                                                  |
| Ctrl+Shift+Z | Redo                                                  |
| Esc          | Opens the menu (The menu is still under development.) |

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.