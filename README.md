
# 📁 IntrRaCLI – Terminal File Explorer in Rust 🦀

IntrRaCLI is a simple yet powerful Terminal User Interface (TUI) file explorer written in Rust. It combines the flexibility of `ratatui` for rendering UI components with the control of `crossterm` for handling terminal interactions. Whether you're browsing, previewing, or managing files directly from your terminal, IntrRaCLI has got your back.

---

## ✨ Features

- 🚀 **Keyboard Navigation** – Use arrow keys to smoothly navigate through files and folders.
- 📂 **Directory Traversal** – Enter and exit directories effortlessly.
- 📄 **File Preview** – View file contents directly within the terminal.
- ⚠️ **Error Handling** – Graceful handling of unreadable files with clear messages.
- 📋 **Copy File (Ctrl+C)** – Copy the selected file’s content with a single shortcut.
- 📥 **Paste File (Ctrl+V)** – Paste the copied file into the current directory.
- 🛑 **Exclude Hidden Files** – Automatically hides dotfiles (like `.gitignore`).
- 📝 **Rename Files/Folders (Ctrl+R)** – Rename items using an intuitive pop-up dialog.

---

## 🛠️ Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/BiMalxMe/IntrRaCLI
cd IntrRaCLI
```

### 2. Run the application

```bash
cargo run
```

---

## 🎮 Controls

| Key            | Action                                             |
|----------------|----------------------------------------------------|
| ↑ / ↓          | Move up or down the file list                      |
| Enter          | Open a folder or preview a file                    |
| b              | Go back to the parent directory                    |
| Ctrl+C         | Copy the currently selected file                   |
| Ctrl+V         | Paste the copied file into the current directory   |
| Ctrl+R         | Rename the selected file or folder                 |
| q              | Quit the application                               |

---

## 📁 Project Structure

- **`main.rs`** – Handles core logic, UI rendering, and event handling.
- **`walkdirfile/mod.rs`** – Contains `get_dir_datas`, which fetches directory entries using `walkdir`, excluding hidden files.

---

## 📦 Dependencies

IntrRaCLI is powered by these awesome Rust crates:

- **`ratatui`** – Elegant UI rendering in the terminal
- **`crossterm`** – Terminal input/output and event management
- **`walkdir`** – Efficient directory traversal
- **`dirs`** – Cross-platform support for standard system directories

Check `Cargo.toml` for exact versions and additional dependencies.

---

## 🙌 Contribute

Pull requests and feedback are welcome! Feel free to fork the repo and open an issue or PR.

---

## 📜 License

This project is licensed under the MIT License.
