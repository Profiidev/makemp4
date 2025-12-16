use ratatui::{
  Frame,
  text::Line,
  widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph},
};

use crate::makemkv::DiskInfo;

pub enum AppState {
  InitLoading,
  DriveList {
    drives: Vec<String>,
    selected: ListState,
  },
  DriveInfoLoading(String),
  DriveInfo {
    drive: String,
    disk_info: DiskInfo,
    selected: ListState,
  },
  Done,
  Exit,
}

impl AppState {
  pub fn new() -> Self {
    Self::InitLoading
  }

  pub fn render(&mut self, frame: &mut Frame) {
    match self {
      Self::InitLoading => {
        frame.render_widget(
          Paragraph::new("Loading drives...").block(Block::bordered().title("Init")),
          frame.area(),
        );
      }
      Self::DriveList { drives, selected } => {
        if drives.is_empty() {
          frame.render_widget(
            Paragraph::new("No drives found").block(Block::bordered().title("Drives")),
            frame.area(),
          );
          return;
        }

        let items: Vec<_> = drives.iter().map(|d| ListItem::new(Line::raw(d))).collect();
        frame.render_stateful_widget(
          List::new(items)
            .block(Block::bordered().title("Drives"))
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(" > "),
          frame.area(),
          selected,
        );
      }
      Self::DriveInfoLoading(drive) => {
        frame.render_widget(
          Paragraph::new(format!("Loading titles for drive {}...", drive))
            .block(Block::bordered().title("Drive Info")),
          frame.area(),
        );
      }
      Self::DriveInfo {
        drive,
        disk_info,
        selected,
      } => {
        let title = format!("Drive: {} Title: {} Titles:", drive, disk_info.title);

        if disk_info.titles.is_empty() {
          frame.render_widget(
            Paragraph::new("No titles found").block(Block::bordered().title(title)),
            frame.area(),
          );
          return;
        }

        let items: Vec<_> = disk_info
          .titles
          .iter()
          .map(|t| {
            ListItem::new(Line::raw(format!(
              "ID: {} Name: {} Size: {} bytes Duration: {} secs",
              t.id, t.name, t.size_bytes, t.duration_secs
            )))
          })
          .collect();

        frame.render_stateful_widget(
          List::new(items)
            .block(Block::bordered().title(title))
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(" > "),
          frame.area(),
          selected,
        );
      }
      _ => (),
    }
  }

  pub fn move_selection_up(&mut self) {
    if let Self::DriveList { selected, .. } | Self::DriveInfo { selected, .. } = self {
      let selected_index = selected.selected().unwrap_or(0);
      if selected_index > 0 {
        selected.select(Some(selected_index - 1));
      }
    }
  }

  pub fn move_selection_down(&mut self) {
    let len = self.len();
    if let Self::DriveList { selected, .. } | Self::DriveInfo { selected, .. } = self {
      let selected_index = selected.selected().unwrap_or(0);
      if selected_index + 1 < len {
        selected.select(Some(selected_index + 1));
      }
    }
  }

  pub fn len(&self) -> usize {
    match self {
      Self::DriveList { drives, .. } => drives.len(),
      Self::DriveInfo { disk_info, .. } => disk_info.titles.len(),
      _ => 0,
    }
  }
}
