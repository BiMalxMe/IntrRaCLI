use std::{ io, path::PathBuf};

use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{
    prelude::CrosstermBackend, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, ListState}, Terminal
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

         let mut list_state = ListState::default();
         list_state.select(Some(app.selected));
         f.render_stateful_widget(list, size, &mut list_state);
   })?;

   if let Event::Key(key) = event::read()? {
      match key.code {
          KeyCode::Char('q') => break,
          KeyCode::Down => app.next(),
          KeyCode::Up => app.previous(),
          //else do nothing
          _ => {}
      }
  }
  }

   disable_raw_mode()?;
   execute!(std::io::stdout(),LeaveAlternateScreen)?;
   
    Ok(())
}
