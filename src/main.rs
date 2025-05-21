use std::{ fs::read_to_string, io, path::PathBuf};

use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{
    layout::{Constraint, Direction, Layout}, prelude::CrosstermBackend, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, Terminal
};

pub mod walkdirfile;
struct App {
    selected: usize,
    data: Vec<PathBuf>,
}
impl App {
    fn new(data: Vec<PathBuf>) -> Self {
        App {
            selected: 0,
            data: data,
        }
    }
    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    fn next(&mut self) {
        if self.selected < self.data.len() - 1 {
            self.selected += 1;
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut filedata = String::new();
    let mut entered = false;
    let filelist = walkdirfile::waldirconfigs::getsrc_files();

    if filelist.is_empty() {
        eprintln!(" No files found ");
        return Ok(());
    }

    let mut app = App::new(filelist);

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop{
    terminal.draw(|f| {
        let size = f.area();
        let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .margin(1)
                .split(size);
     
        
        let block = Block::default().title("File").borders(Borders::all());
        let listitems: Vec<ListItem> = app
            .data
            .iter()
            .map(|path| ListItem::new(path.display().to_string()))
            .collect();

         let list = List::new(listitems)
         .block(block)
         .highlight_symbol("-> ")
         .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Blue));
        
        if entered{

            let selectedfile = &app.data[app.selected];

            let readfile = read_to_string(selectedfile); 
            match readfile {
                Ok(data) => filedata = data,
                Err(e) => eprint!("An error Occurred {}",e)
            }
            let para = Paragraph::new(filedata.clone())
            .block(Block::default()
            .title(selectedfile.display().to_string().replace("./", "")) // ✅ Convert PathBuf to String
            .borders(Borders::all()))
            .style(Style::default().bg(Color::Magenta).fg(Color::Blue));

            f.render_widget(para, chunks[1]);
        }

         let mut list_state = ListState::default();
         list_state.select(Some(app.selected));
         f.render_stateful_widget(list, chunks[0], &mut list_state);
   })?;

   if let Event::Key(key) = event::read()? {
      match key.code {
          KeyCode::Char('q') => break,
          KeyCode::Down => app.next(),
          KeyCode::Up => app.previous(),
          KeyCode::Enter => {
           entered = true;
          }
          _ => {}
      }
  }
  }

   disable_raw_mode()?;
   execute!(std::io::stdout(),LeaveAlternateScreen)?;
   
    Ok(())
}
