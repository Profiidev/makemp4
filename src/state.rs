use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout},
  text::Line,
  widgets::{Block, Gauge, HighlightSpacing, List, ListItem, ListState, Paragraph},
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
  TitleExtracting {
    drive: String,
    disk_info: DiskInfo,
    title_id: u32,
    total: u32,
    extracted: u32,
    task: String,
  },
  Done {
    disk_info: DiskInfo,
    drive: String,
    title_id: u32,
    selected: ListState,
  },
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
            Paragraph::new("No drives found\nPress Enter to retry")
              .block(Block::bordered().title("Drives")),
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
            Paragraph::new("No titles found\nPress Enter to retry")
              .block(Block::bordered().title(title)),
            frame.area(),
          );
          return;
        }

        let items: Vec<_> = disk_info
          .titles
          .iter()
          .map(|t| {
            ListItem::new(Line::raw(format!(
              "ID: {} Name: {} Size: {:.2} GB Duration: {} mins",
              t.id,
              t.name,
              t.size_bytes as f64 / 1_000_000_000.0,
              t.duration_secs / 60
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
      Self::TitleExtracting {
        drive,
        title_id,
        total,
        extracted,
        task,
        ..
      } => {
        let percentage = if *total > 0 {
          (*extracted as f64 / *total as f64) * 100.0
        } else {
          0.0
        };

        let block = Block::bordered().title("Extracting Title");
        let area = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        let layout = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Length(3), Constraint::Max(1)])
          .split(area);

        frame.render_widget(
          Paragraph::new(format!(
            "Extracting title {} from drive {}...\n{}\nProgress: {}/{}",
            title_id, drive, task, extracted, total
          )),
          layout[0],
        );

        frame.render_widget(
          Gauge::default()
            .ratio(percentage / 100.0)
            .label(format!("{:.2}%", percentage))
            .use_unicode(true),
          layout[1],
        );
      }
      Self::Done {
        drive,
        title_id,
        selected,
        ..
      } => {
        let block = Block::bordered().title("Done");
        let area = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        let layout = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Length(1), Constraint::Length(2)])
          .split(area);

        frame.render_widget(
          Paragraph::new(format!(
            "Finished extracting title {} from drive {}.",
            title_id, drive
          )),
          layout[0],
        );

        let items = vec![
          ListItem::new(Line::raw("Exit")),
          ListItem::new(Line::raw("Extract Another Title")),
        ];

        frame.render_stateful_widget(
          List::new(items)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(" > "),
          layout[1],
          selected,
        );
      }
      Self::Exit => (),
    }
  }

  pub fn move_selection_up(&mut self) {
    if let Self::DriveList { selected, .. }
    | Self::DriveInfo { selected, .. }
    | Self::Done { selected, .. } = self
    {
      let selected_index = selected.selected().unwrap_or(0);
      if selected_index > 0 {
        selected.select(Some(selected_index - 1));
      }
    }
  }

  pub fn move_selection_down(&mut self) {
    let len = self.len();
    if let Self::DriveList { selected, .. }
    | Self::DriveInfo { selected, .. }
    | Self::Done { selected, .. } = self
    {
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
      Self::Done { .. } => 2,
      _ => 0,
    }
  }
}
