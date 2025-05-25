use std::io::Write;
use std::{
    fs::{File, read_to_string},
    io,
    path::PathBuf,
};

use crossterm::event::KeyModifiers;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub mod walkdirfile;
struct CopyFiles {
    data: Option<String>,
    filename: Option<String>,
}

impl CopyFiles {
    // for inintailizing with empty paramts for the top level flobal declaration
    pub fn new() -> Self {
        CopyFiles {
            data: None,
            filename: None,
        }
    }

    //update the self with the main data
    pub fn update_all(&mut self, new_data: String, new_filename: Option<PathBuf>) {
        let filepath: Option<String> = new_filename.and_then(|path_buf| {
            path_buf
                .file_name()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_string())
        });

        // assigns the new data, wrapping it in Some
        self.data = Some(new_data);

        // assigns the new filename
        self.filename = filepath; // filepath is already Option<String>
    }
}

struct App {
    selected: usize,
    data: Vec<(String, PathBuf)>,
    currentpath: PathBuf,
    selectedfile: Option<PathBuf>,
    dialogueboxappear: bool,
    renamedinput: String,
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
            //added some fields for the reneme logic
            dialogueboxappear: false,
            renamedinput: String::new(),
        }
    }
    //for toggling the value of the
    fn toggle_dialog(&mut self) {
        //while toggle is called use negation of the value
        self.dialogueboxappear = !self.dialogueboxappear;
        if !self.dialogueboxappear {
            self.renamedinput.clear();
        }
    }
    fn handle_dialog_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.renamedinput.push(c);
            }
            KeyCode::Backspace => {
                self.renamedinput.pop();
            }

            //if enter is pressed when the inner rename box is opened
            KeyCode::Enter => {
                //if the renamedinput is not empty
                if !self.renamedinput.is_empty() {
                    //get the oldpath from the vec in data of the selected
                    if let Some((_, old_path)) = self.data.get(self.selected) {
                        // Extract file extension
                        let extension = old_path.extension().and_then(|ext| ext.to_str());

                        // Check if the renamed input already contains an extension
                        let new_path = {
                            //get the renamed file name prefix of the .extension
                            let mut new_filename = self.renamedinput.clone();

                            //if the user is not giving a extension while renaming
                            // then you should use the extension as the old path had
                            if !new_filename.contains('.') {
                                //get the extension
                                if let Some(ext) = extension {
                                    //add the . after the newfilname renmaed
                                    new_filename.push('.');

                                    //push the extension
                                    new_filename.push_str(ext);
                                }
                            }
                            //this created the filename with the same path but with differnt filename

                            //for E.g
                            //
                            // let old_path = Path::new("/home/user/documents/old_name.txt");
                            // let new_path = old_path.with_file_name("new_name.txt");

                            // println!("{:?}", new_path);
                            // Output: "/home/user/documents/new_name.txt"
                            old_path.with_file_name(new_filename)
                        };

                        //change old path to new path
                        match std::fs::rename(old_path, &new_path) {
                            Ok(_) => {
                                //get the index where the list is hovered upon
                                let current_index = self.selected;

                                //update and get the new filelist from walkdir
                                self.update_app();

                                //after new renmaed update
                                //change selected to previoduly renamed file index
                                
                                //max is currentindex and min is final index of the data
                                self.selected =
                                    current_index.min(self.data.len().saturating_sub(1));

                                //that new path file should be selected 
                                self.selectedfile = Some(new_path);
                            }
                            Err(e) => {
                                //if errror occurs
                                eprintln!("Error renaming file: {}", e);
                            }
                        }
                    }
                }
                self.toggle_dialog();
            }
            _ => {}
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
    let mut app = App::new(
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .into(),
    );

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
    let mut copyinstance = CopyFiles::new();

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

            if app.dialogueboxappear {
                // Define the dialog block with title, borders, and background color
                let block = Block::default()
                    .title(" Rename Dialog (ESC to close) ")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::DarkGray));

                // Prepare the text content for the paragraph
                let text = Text::from(vec![
                    Line::from(" Type your file/folder name to rename:"),
                    Line::from(""),
                    Line::from(format!(" > {}", app.renamedinput)),
                    Line::from(""),
                    Line::from(" Backspace: Delete    |    ESC: Close"),
                ]);

                let paragraph = Paragraph::new(text).block(block);

                // Center the dialog in the terminal
                let area = centered_rect(60, 25, size);
                f.render_widget(Clear, area); // Clear background for transparency
                f.render_widget(paragraph, area);
            }
        })?;
        if let Event::Key(key) = event::read()? {
            if app.dialogueboxappear {
                match key.code {
                    KeyCode::Esc => app.toggle_dialog(),
                    //othre key event are handled in the handle_dialog_input fn in implementation
                    _ => app.handle_dialog_input(key),
                }
                continue;
            }
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), KeyModifiers::NONE) => break,
                (KeyCode::Down, KeyModifiers::NONE) => app.next(),
                (KeyCode::Up, KeyModifiers::NONE) => app.previous(),
                // set the global entered true such that the file content or the error displays
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    entered = true;
                }
                (KeyCode::Char('b'), KeyModifiers::NONE) => {
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
                                    copyinstance.update_all(content, Some(path.clone()));
                                }
                                Err(e) => {
                                    eprintln!("Failed to read file: {}", e);
                                }
                            }
                        } else {
                            eprintln!("Cannot copy directory content");
                        }
                    }
                }
                // Ctrl+R implementation
                (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                    app.toggle_dialog();
                }
                // on clicking the ctrl
                (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                    //get the filename and content from the file we copied from
                    //can see the logic in the ctrl+c

                    if let (Some(filename), Some(content)) =
                        (&copyinstance.filename, &copyinstance.data)
                    {
                        //get the current path where we want to paste
                        // cloning it such that old path shouldnot be changes
                        //for eg
                        //new_path becomes something like /current/directory/filename.txt
                        // app.currentpath remains /current/directory
                        let mut new_path = app.currentpath.clone();
                        new_path.push(filename);

                        //pattern matching
                        match File::create(&new_path) {
                            //if file exist and ther is no error
                            Ok(mut file) => {
                                //This code tries to write content to a file and checks
                                // for errors. If it fails, it prints an error message to stderr.
                                if let Err(e) = file.write_all(content.as_bytes()) {
                                    eprintln!("Failed to write to file: {}", e);
                                }
                                //after writing update and get the new waldir filelist
                                app.update_app();
                            }
                            //if error printit
                            Err(e) => {
                                eprintln!("Failed to create file: {}", e);
                            }
                        }
                    } else {
                        //if something fails print
                        eprintln!("No file content to paste.");
                    }
                }

                //handle other innecessary keypress
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
fn centered_rect(
    //   Width of the centered area as a percentage of the total width
    percent_x: u16,

    // Height of the centered area as a percentage of the total height
    percent_y: u16,

    // The full outer rectangle (usually the entire terminal area)
    r: ratatui::prelude::Rect,
) -> ratatui::prelude::Rect {
    // First, split the outer rectangle vertically into 3 parts:
    // Top margin, center area, and bottom margin
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2), // Top margin
            Constraint::Percentage(percent_y),             // Center height
            Constraint::Percentage((100 - percent_y) / 2), // Bottom margin
        ])
        // apply the vertical layout split on the input rect
        .split(r);

    //  split the center vertical area horizontally into 3 parts:
    // Left margin, center , and right widths
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2), // Left
            Constraint::Percentage(percent_x),             // Center
            Constraint::Percentage((100 - percent_x) / 2), // Right
        ])
        //means
        // "Take the middle vertical section, split it horizontally, and return the middle horizontal section."
        .split(popup_layout[1])[1]
}
