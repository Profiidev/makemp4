use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::ListState};
use std::{
  sync::{Arc, Mutex},
  thread::spawn,
  time::Duration,
};

use crate::{
  makemkv::{self, DiskInfo},
  state::AppState,
};

pub struct App {
  state: Arc<Mutex<AppState>>,
}

impl App {
  pub fn new() -> Self {
    let state = Arc::new(Mutex::new(AppState::new()));
    spawn({
      let state = state.clone();
      move || {
        let drives = makemkv::find_drives().unwrap_or_default();
        let mut state = state.lock().unwrap();
        *state = AppState::DriveList {
          drives,
          selected: ListState::default().with_selected(Some(0)),
        };
      }
    });

    Self { state }
  }

  pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
    loop {
      let mut state = self.state.lock().unwrap();
      if matches!(*state, AppState::Exit) {
        break;
      }
      terminal.draw(|frame| state.render(frame))?;
      drop(state);
      self.handle_crossterm_events()?;
    }

    Ok(())
  }

  pub fn handle_crossterm_events(&mut self) -> Result<()> {
    if !event::poll(Duration::from_millis(100))? {
      return Ok(());
    }

    match event::read()? {
      Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
      Event::Mouse(_) => {}
      Event::Resize(_, _) => {}
      _ => (),
    }
    Ok(())
  }

  fn on_key_event(&mut self, key: KeyEvent) {
    match key.code {
      KeyCode::Char('q') | KeyCode::Esc => {
        let mut state = self.state.lock().unwrap();
        *state = AppState::Exit;
      }
      KeyCode::Up | KeyCode::Char('k') => {
        let mut state = self.state.lock().unwrap();
        state.move_selection_up();
      }
      KeyCode::Down | KeyCode::Char('j') => {
        let mut state = self.state.lock().unwrap();
        state.move_selection_down();
      }
      KeyCode::Enter | KeyCode::Char('l') => {
        let mut state = self.state.lock().unwrap();
        if let AppState::DriveList { drives, selected } = &*state
          && let Some(index) = selected.selected()
          && let Some(drive) = drives.get(index)
        {
          let drive_clone = drive.clone();
          *state = AppState::DriveInfoLoading(drive_clone.clone());
          let state_arc = self.state.clone();
          spawn(move || {
            let disk_info = makemkv::find_disk_titles(&drive_clone).unwrap_or(DiskInfo {
              title: String::new(),
              titles: Vec::new(),
            });
            let mut state = state_arc.lock().unwrap();
            *state = AppState::DriveInfo {
              drive: drive_clone,
              disk_info,
              selected: ListState::default().with_selected(Some(0)),
            };
          });
        }
      }
      _ => {}
    }
  }
}
