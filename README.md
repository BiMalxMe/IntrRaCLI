# Rust TUI File Explorer

This is a terminal user interface (TUI) file explorer written in Rust, utilizing `crossterm` for terminal manipulation and `ratatui` for UI rendering.

---

## Features

* **Navigation:** Browse directories and files using arrow keys.
* **Directory Traversal:** Enter directories to view their contents, and go back to parent directories.
* **File Viewing:** Select a file to display its content within the terminal.
* **Error Handling:** Displays error messages if a file cannot be read.
* **Copy File Content (Ctrl+C):** Copy the content of a selected file.
* **Hidden File Exclusion:** Automatically hides dotfiles (files starting with `.`).

---

## Installation and Running

1.  **Clone the repository:**

    ```bash
    git clone <repository_url> # Replace <repository_url> with the actual URL
    cd <repository_name>
    ```

2.  **Run the application:**

    ```bash
    cargo run
    ```

---

## Usage

* **`Up Arrow` / `Down Arrow`:** Navigate through the file and directory list.
* **`Enter`:**
    * If a directory is selected, enter that directory.
    * If a file is selected, display its content.
* **`b`:** Go back to the parent directory.
* **`Ctrl+C`:** Copy the content of the currently selected file. (Note: This currently only prints to the console that the content has been copied, it doesn't paste to your system clipboard.)
* **`q`:** Quit the application.

---

## Code Structure

* **`main.rs`:** Contains the main application logic, TUI rendering, and event handling.
* **`walkdirfile/mod.rs` (or `walkdirfile/waldirconfigs.rs`):** Contains the `get_dir_datas` function, which uses the `walkdir` crate to list directory contents, excluding hidden files.

---

## Dependencies

This project relies on the following Rust crates:

* **`crossterm`**: For terminal event handling and low-level terminal manipulation.
* **`ratatui`**: For building the terminal user interface.
* **`walkdir`**: For efficiently traversing directory trees.
* **`dirs`**: For cross-platform handling of user directories (e.g., home directory).

You can find these dependencies listed in `Cargo.toml`.