use std::fs;
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
use ratatui::layout::Alignment;
use ratatui::style::Modifier;
use ratatui::text::Span;
use ratatui::widgets::BorderType;
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

enum FileType {
    File,
    Folder,
    None
}

struct App {
    selected: usize,
    data: Vec<(String, PathBuf)>,
    currentpath: PathBuf,
    selectedfile: Option<PathBuf>,
    dialogueboxappear: bool,
    renamedinput: String,
    // New field for scroll position
    scroll: u16,
    filesmaxlines : u16,
    wannadelete : bool,
    deletedsucessfully : bool,
    wannacreate : bool,
    creationtype : FileType
}

//adding this will allow us to add logical syntaxes
#[derive(PartialEq)]
enum FileDialogSection {
    Filename,
    Content,
}

struct Newcontent {
    filename: Option<String>,
    filecontent: Option<String>,
    foldername: Option<String>,
    currentinput: String,
    active_section: FileDialogSection,  // Add this field
    filename_input: String,  // Separate field for filename input
    content_input: String,   // Separate field for content input
}

impl Newcontent {
    fn new() -> Self {
        Newcontent {
            filename: None,
            filecontent: None,
            foldername: None,
            currentinput: String::new(),
            active_section: FileDialogSection::Filename,
            filename_input: String::new(),
            content_input: String::new(),
        }
    }
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
            scroll: 0,
            filesmaxlines: 0,
            wannadelete : false,
            deletedsucessfully : false,
            wannacreate : false,
            creationtype : FileType::None
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
    fn toggle_delete(&mut self){
        self.wannadelete = !self.wannadelete;
    }
    fn handle_delete(&mut self, key: KeyEvent){
        match key.code {
            KeyCode::Enter => {
                //get the current selected path
                if let Some((_, path)) = self.data.get(self.selected) {
                    // if path is a directory
                    if path.is_dir() {
                        //removes folder and its contents
                        match fs::remove_dir_all(path) {
                            Ok(_) => {
                                self.deletedsucessfully = true;
                            }
                            Err(e) => {
                                eprintln!("Error deleting directory '{:?}': {}", path, e);
                            }
                        }
                        //if the path is file type
                    } else if path.is_file() {
                        match fs::remove_file(path) {
                            Ok(_) => {
                                self.deletedsucessfully = true;
                            }
                            Err(e) => {
                                eprintln!("Error deleting file '{:?}': {}", path, e);
                            }
                        }
                    } else {
                        //else cannot
                        eprintln!("Cannot delete: '{:?}' is not a file or directory.", path);
                    }
                } else {
                    //if path is not valid 
                    eprintln!("No item selected for deletion.");
                }
            }
            _ => {}
        }
        if self.deletedsucessfully{
            // change the selection from deleted path to none
            self.selectedfile = None;
            self.deletedsucessfully = false;

        }
        self.update_app();
        self.toggle_delete();
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

    // Scroll up by one line
    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    // Scroll down by one line
    fn scroll_down(&mut self) {
        // Prevent scrolling beyond the content
        if self.scroll < self.filesmaxlines.saturating_sub(1) {
            self.scroll += 1;
        }
    }

    // Reset scroll when opening a new file
    fn reset_scroll(&mut self) {
        self.scroll = 0;
    }
    
    //logic for creating a new folder/file
    fn toggle_creation(&mut self){
        self.wannacreate = !self.wannacreate;
    }
    fn handle_creation(&mut self,key: KeyEvent){
        
        match key.code {
        // if  m or f is pressed the dialogue box of creation diappers
            KeyCode::Char('m') => {
                //folder logic
               self.creationtype = FileType::Folder;
               self.toggle_creation();

            }
            KeyCode::Char('f') => {
                //creation of file logic
                self.creationtype = FileType::File;
                self.toggle_creation();
            }
            _ => {}
        }
    }

}

fn main() -> std::io::Result<()> {
    let mut create_new_file_and_folder = Newcontent::new();
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
    if app.deletedsucessfully{
        filedata.clear();
        app.deletedsucessfully = false;
    }
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
        
                // Check if file exists before trying to read
                if !path.exists() {
                    error_message = Some("File no longer exists".to_string());
                    filedata.clear();
                    app.selectedfile = None;
                } else {
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
            let filenameonly = if let Some(path) = &app.selectedfile {
                path.to_string_lossy().into_owned()
            } else {
                filedata.clear();
                "No file selected".to_string()
            };
            // file ko line
            let content_lines = filedata.lines().count() as u16;
            // Terminal ko size
            let content_area_height = chunks[1].height.saturating_sub(2); // Subtract 2 for top/bottom borders

            app.filesmaxlines = content_lines;
            // Update scroll_down boundary
            // can be zero or the value (excedding lines below terminal)
            let max_scroll = content_lines.saturating_sub(content_area_height);
            if app.scroll > max_scroll {
                app.scroll = max_scroll;
            }

            // Make a para of the file content such that the filecontent gets good visual design and all
            let para = Paragraph::new(filedata.clone())
                .block(
                    Block::default()
                        .title(format!(" 📜 {} (Shift + Up/Down to scroll) ", filenameonly)), // Updated instructions
                )
                .style(Style::default().bg(Color::DarkGray).fg(Color::Blue))
                .scroll((app.scroll, 0));

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
            if app.wannadelete {
                let block = Block::default()
                    .title(" Delete Confirmation (ESC to close) ")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::DarkGray));
            
                let dialog = Paragraph::new("⚠️  Are you sure you want to delete this file?\nPress Enter to confirm, 'Esc' to cancel.")
                    .block(block)
                    .style(Style::default().bg(Color::Black).fg(Color::Red));
            
                // draw the popup in the center of the screen
                let area = centered_rect(40, 16, size);
                f.render_widget(Clear, area); // clear underlying widgets
                f.render_widget(dialog, area);
            }
            
            if app.wannacreate {

                let block = Block::default()
                    .title(Span::styled(" New Creation ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))) // Title text color and bold
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded) // Example: Rounded borders
                    .style(Style::default().bg(Color::DarkGray)); // A more subtle background for the block
            
                let dialog = Paragraph::new("Enter \"f\" for creating file and \"m\" for creating folder.")
                    .alignment(Alignment::Center) // Center the text within the paragraph
                    .block(block)
                    .style(Style::default().fg(Color::LightCyan).bg(Color::DarkGray)); // LightCyan text on DarkGray background
            
                // draw the popup in the center of the screen
                let area = centered_rect(60, 20, size);
                f.render_widget(Clear, area); // clear underlying widgets
                f.render_widget(dialog, area);
            }
            match app.creationtype {
               FileType::None => {},
               FileType::File => {
                let area = centered_rect(40, 50, size);
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                    .margin(1)
                    .split(area);
            
                // Filename box with conditional styling
                let filename_block = Block::default()
                    .title(" Filename (Tab to switch) ")
                    .borders(Borders::ALL)
                    .style(if create_new_file_and_folder.active_section == FileDialogSection::Filename {
                        Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::Yellow)
                    } else {
                        Style::default().bg(Color::Black).fg(Color::Gray)
                    });
            
                let upperone = Paragraph::new(create_new_file_and_folder.filename_input.as_str())
                    .block(filename_block);
            
                // Content box with conditional styling
                let content_block = Block::default()
                    .title(" Content (Tab to switch) ")
                    .borders(Borders::ALL)
                    .style(if create_new_file_and_folder.active_section == FileDialogSection::Content {
                        Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::Yellow)
                    } else {
                        Style::default().bg(Color::Black).fg(Color::Gray)
                    });
            
                let lowerone = Paragraph::new(create_new_file_and_folder.content_input.as_str())
                    .block(content_block);
            
                f.render_widget(Clear, area);
                f.render_widget(upperone, chunks[0]);
                f.render_widget(lowerone, chunks[1]);
            }
               FileType::Folder => {
                let block = Block::default()
                .title(Span::styled(" Create a Folder ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))) // Title text color and bold
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded) // Example: Rounded borders
                .style(Style::default().bg(Color::DarkGray)); 

                  
            let dialog: Paragraph<'_> = Paragraph::new(create_new_file_and_folder.currentinput.as_str())
                .alignment(Alignment::Center) // Center the text within the paragraph
                .block(block)
                .style(Style::default().fg(Color::White).bg(Color::DarkGray)); // LightCyan text on DarkGray background
        
            // draw the popup in the center of the screen
            let area = centered_rect(30, 20, size);
            f.render_widget(Clear, area); // clear underlying widgets
            f.render_widget(dialog, area);
               },
            }

        })?;
        if let Event::Key(key) = event::read()? {
            match app.creationtype{
                FileType::Folder => {
                    match key.code {
                        //if anykey is pressed
                        KeyCode::Char(c) => {
                            //push into the global string catcher
                            create_new_file_and_folder.currentinput.push(c);
                            continue;
                        }
                        KeyCode::Backspace => {
                            //if backspace then remove the last letter from the global string
                            create_new_file_and_folder.currentinput.pop();
                            continue;
                        }
                        //if entered thenn
                        KeyCode::Enter => {
                            //if not empty
                            if !create_new_file_and_folder.currentinput.is_empty() {
                                //get the currentpath where ctrl n is pressed
                                // its like users/bimal/all/
                                let mut new_folder_path = app.currentpath.clone();

                                //add the foldername in the new_folder path
                                // if newfoldername is rust
                                //then new_folder_path will be users/bimal/all/rust
                                new_folder_path.push(&create_new_file_and_folder.currentinput);
                                
                                //creates the directory in the path modified
                                match fs::create_dir(&new_folder_path) {
                                    Ok(_) => {
                                        //after change in the folder structure , I reloads the waldir configs
                                        app.update_app();
                                        app.creationtype = FileType::None;
                                        //flags to default
                                        app.wannacreate = false;
                                        // insert into the structs foldername element (Optional not need right now)
                                        create_new_file_and_folder.foldername = Some(create_new_file_and_folder.currentinput.clone());
                                        //clears the input
                                        create_new_file_and_folder.currentinput.clear();
                                    }
                                    Err(e) => {
                                        error_message = Some(format!("Failed to create folder: {}", e));
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.creationtype = FileType::None;
                            app.wannacreate = false;
                            create_new_file_and_folder.currentinput.clear();
                            continue;
                        }
                        _ => {}
                    }
                }
                FileType::File => {
                    match key.code {
                        // Tab switches between filename and content sections
                        KeyCode::Tab => {
                            create_new_file_and_folder.active_section = match create_new_file_and_folder.active_section {
                                FileDialogSection::Filename => FileDialogSection::Content,
                                FileDialogSection::Content => FileDialogSection::Filename,
                            };
                        }
                        // Handle input based on active section
                        KeyCode::Char(c) => {
                            match create_new_file_and_folder.active_section {
                                FileDialogSection::Filename => {
                                    create_new_file_and_folder.filename_input.push(c);
                                    continue;

                                }
                                FileDialogSection::Content => {
                                    create_new_file_and_folder.content_input.push(c);
                                    continue;

                                }
                            }
                        }
                        KeyCode::Backspace => {
                            match create_new_file_and_folder.active_section {
                                //for filename input
                                FileDialogSection::Filename => {
                                    create_new_file_and_folder.filename_input.pop();
                                }
                                //for filecontent input
                                FileDialogSection::Content => {
                                    create_new_file_and_folder.content_input.pop();
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if !create_new_file_and_folder.filename_input.is_empty() {
                                //only if the path is file
                                let mut new_file_path = if app.currentpath.is_file() {
                                    app.currentpath.parent().unwrap_or(&app.currentpath).to_path_buf()
                                } else {
                                    //else normal
                                    app.currentpath.clone()
                                };
                                //make a pathbuf including the filename
                                new_file_path.push(&create_new_file_and_folder.filename_input);
                                
                                //make a file with the cintent inside
                                match File::create(&new_file_path) {
                                    Ok(mut file) => {
                                        if !create_new_file_and_folder.content_input.is_empty() {
                                            //if the user had inputed the content too then write inside the made file
                                            file.write_all(create_new_file_and_folder.content_input.as_bytes()).ok();
                                        }
                                        app.update_app();
                                        app.creationtype = FileType::None;
                                        app.wannacreate = false;
                                        create_new_file_and_folder = Newcontent::new();
                                    }
                                    Err(e) => error_message = Some(format!("Failed to create file: {}", e)),
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.creationtype = FileType::None;
                            app.wannacreate = false;
                            create_new_file_and_folder = Newcontent::new();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            if app.dialogueboxappear {
                match key.code {
                    KeyCode::Esc => app.toggle_dialog(),
                    //othre key event are handled in the handle_dialog_input fn in implementation
                    _ => app.handle_dialog_input(key),
                }
                continue;
            }
            //if the wannadelete is true
            if app.wannadelete {
                match key.code {
                    //if Esc then close it 
                    KeyCode::Esc => app.toggle_delete(),
                    //else transfer the other keyevent handler in handle_delete function
                    _ => app.handle_delete(key),
                }
                continue;
            }
            if app.wannacreate{
                match key.code {
                    //exit the dialogue box for  the creation
                   KeyCode::Esc => app.wannacreate = false,
                   //else
                   _ => app.handle_creation(key), 
                }
            }
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), KeyModifiers::NONE) => break,
                (KeyCode::Down, KeyModifiers::NONE) => app.next(),
                (KeyCode::Up, KeyModifiers::NONE) => app.previous(),
                // set the global entered true such that the file content or the error displays
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    entered = true;
                    app.reset_scroll();
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
                        app.reset_scroll();
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
                (KeyCode::Char('n'),KeyModifiers::CONTROL) => {
                    //do something
                    app.toggle_creation();
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
                        //if something fails printf
                        eprintln!("No file content to paste.");
                    }
                }
                (KeyCode::Up, KeyModifiers::SHIFT) => {
                    app.scroll_up();
                }
                (KeyCode::Down, KeyModifiers::SHIFT) => {
                    if app.selectedfile.is_some() && error_message.is_none() {
                        app.scroll_down();
                    }
                }
                (KeyCode::Backspace, KeyModifiers::NONE) if !app.wannadelete => {
                    app.toggle_delete();
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
