use std::{fs::read_to_string, io, path::PathBuf};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
     layout::{Constraint, Direction, Layout}, prelude::CrosstermBackend, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, Terminal
};
use crossterm::event::KeyModifiers;


pub mod walkdirfile;

struct App {
    selected: usize,
    data: Vec<(String,PathBuf)>,
    currentpath: PathBuf,
    selectedfile: Option<PathBuf>,
}

//implemeting the steuct to get functions like prev and next and new
impl App {
    fn new(initial_path: PathBuf) -> Self {
        let default_path = if initial_path.as_os_str().is_empty() {
            PathBuf::from("./") // Default to current directory if initial_path is empty
        } else {
            initial_path
        };

        // Populate data using the default_path
        let data = walkdirfile::waldirconfigs::get_dir_datas(default_path.clone());

        App {
            selected: 0,
            //data taken from the path
            data,
            currentpath: default_path,
            selectedfile: None,
        }
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn next(&mut self) {
        if self.selected < self.data.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn update_app(&mut self) {
        //get the file from the waldirfile.rs
        self.data = walkdirfile::waldirconfigs::get_dir_datas(self.currentpath.clone());
        self.selected = 0;
    }
}

fn main() -> std::io::Result<()> {
    // Contents of the file
    let mut filedata = String::new();
    //Only on the first enter it displays the datas
    let mut entered: bool = false;
    let mut error_message: Option<String> = None;
    //create a new instance of the struct with own vecs
    // use dir crate to go to the main path of the mac machine
    let mut app = App::new(dirs::home_dir().unwrap_or_else(|| PathBuf::from("./")).into());

    let filelist = app.data.clone();

    // if the file has no content then print the error
    if filelist.is_empty() {
        eprintln!(" No files found ");
        return Ok(());
    }

    // for efficiency of the keyword
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    // take the whole logic into the seperate terminal
    execute!(stdout, EnterAlternateScreen)?;
    // Create the terminal with standard output system
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let copyinstance = CopyFiles::new();

    // looping such that the value can be hold
    loop {
        // Handle directory change BEFORE rendering
        if entered {
            if let Some((_, path)) = app.data.get(app.selected) {
                if path.is_dir() {
                    app.currentpath = path.clone();
                    app.update_app();
                    entered = false;
                    continue;
                }

                // File case: Try to read
                match read_to_string(path) {
                    Ok(content) => {
                        app.selectedfile = Some(path.clone());
                        filedata = content;
                        error_message = None;
                    }
                    Err(e) => {
                        error_message = Some(format!("Error reading file: {}", e));
                        filedata.clear();
                    }
                }
            }
            entered = false;
        }

        terminal.draw(|f| {
            let size = f.area();

            // the main block which divides the two block of filelist and file contents horizontally
            //such that they take the space accordingly
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .margin(1)
                .split(size);

            // Just a random block
            let block = Block::default().title("File").borders(Borders::all());

            // convert the vector elemtns to the listitem as they can be further used in the list
            let listitems: Vec<ListItem> = app
                .data
                .iter()
                // for osstr to string we use the string lossy
                .map(|(label, _)| ListItem::new(label.clone()))
                .collect();

            // using those formatted listitem inside the list ot give the proper listing
            let list = List::new(listitems)
                .block(block)
                // Appears in front of the file
                .highlight_symbol("-> ")
                .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Blue));

            // extract the filename from the pathBuf
            let filenameonly: String = app.selectedfile.as_ref().map_or_else(
                || "No file selected".to_string(), // If app.selectedfile is None
                |path_buf| path_buf.to_string_lossy().into_owned(), // If Some(path_buf), convert to owned String
            );

            // Make a para of the file content such that the filecontent gets good visual design and all
            let para = Paragraph::new(filedata.clone())
                .block(Block::default().title(filenameonly).borders(Borders::all()))
                .style(Style::default().bg(Color::Magenta).fg(Color::Blue));

            // that error stored by the error_message is now being handled
            // if the error is there then there is no possibility for displaying the content
            // so if the error occurrs then replacing the filecontent rendering logic with
            // the error message -> Like the error is shown as a content with title Error when
            // the error occcurs 
            // Such that the error could be handled easily and they can be seen in the terminal everywjere 
            // and just putting the error into its own section 
            //Either error or filecontent can be shown at a time

            if let Some(err) = &error_message {
                let error_block = Paragraph::new(err.as_str())
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().title("Error").borders(Borders::ALL));
                // if error return error showing teminal box
                f.render_widget(error_block, chunks[1]);
            } else {
                // if the error is not there then shos the content
                f.render_widget(para, chunks[1]);
            }

            // should also give the state as it is a dynamic as selected moves
            let mut list_state = ListState::default();

            // getting the currently selected 
            list_state.select(Some(app.selected));

            // rendering
            f.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;
        if let Event::Key(key) = event::read()? {
            match (key.code,key.modifiers) {
                (KeyCode::Char('q'),KeyModifiers::NONE) => break,
                (KeyCode::Down,KeyModifiers::NONE) => app.next(),
                (KeyCode::Up,KeyModifiers::NONE) => app.previous(),
                // set the global entered true such that the file content or the error displays
                (KeyCode::Enter,KeyModifiers::NONE) => {
                    entered = true;
                }
                (KeyCode::Char('b'),KeyModifiers::NONE) => {
                    //we should use the parent() to find the files parent
                    // if the parent path exist then it rns
                    if let Some(parent) = app.currentpath.parent() {
                        // current path will be the parents path
                        // convert into th pathbuf as the currentpath expects
                        app.currentpath = parent.to_path_buf();

                        //update the apps latest changes
                        app.update_app();
                    }
                }
                 
        // Ctrl+C implementation
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            if let Some((_, path)) = app.data.get(app.selected) {
                if path.is_file() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            copyinstance.update_all(content, path.to_path_buf());
                            println!("Copied file content: {} bytes", content.len());
                        },
                        Err(e) => {
                            eprintln!("Failed to read file: {}", e);
                        }
                    }
                } else {
                    eprintln!("Cannot copy directory content");
                }
            }
        },
       
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    // leave the screen once the q is pressed
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    // return the successfull executoin msg
    Ok(())
}
