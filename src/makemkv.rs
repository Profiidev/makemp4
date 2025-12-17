use std::{
  fs,
  io::{BufRead, BufReader},
  process::{Command, Stdio},
  sync::{Arc, Mutex},
};

use color_eyre::eyre::{Result, bail};
use ratatui::widgets::ListState;

use crate::state::AppState;

pub fn find_drives() -> Result<Vec<String>> {
  let entries = fs::read_dir("/dev")?;
  let mut drives = Vec::new();
  for entry in entries {
    let entry = entry?;
    let file_name = entry.file_name();
    let file_name_str = file_name.to_string_lossy();
    if file_name_str.starts_with("sr") && file_name_str.len() >= 3 {
      drives.push(format!("/dev/{}", file_name_str));
    }
  }
  Ok(drives)
}

#[allow(unused)]
struct CInfo {
  id: u32,
  code: u32,
  value: String,
}

#[allow(unused)]
struct TInfo {
  id: u32,
  code: u32,
  some_id: u32,
  value: String,
}

#[derive(Clone)]
pub struct DiskInfo {
  pub title: String,
  pub titles: Vec<DiskTitle>,
}

#[derive(Clone)]
pub struct DiskTitle {
  pub id: u32,
  pub name: String,
  pub size_bytes: u64,
  pub duration_secs: u32,
}

pub fn find_disk_titles(drive: &str) -> Result<DiskInfo> {
  let output = Command::new("makemkvcon")
    .args(["-r", "info", format!("dev:{}", drive).as_str()])
    .output()?;
  let stdout = String::from_utf8_lossy(&output.stdout);
  let mut cinfos = Vec::new();
  let mut tinfos = Vec::new();

  for line in stdout.lines() {
    if line.starts_with("CINFO:") {
      let parts: Vec<&str> = line
        .strip_prefix("CINFO:")
        .unwrap_or("")
        .split(',')
        .collect();
      if parts.len() >= 3 {
        let id = parts[0].parse::<u32>().unwrap_or(0);
        let code = parts[1].parse::<u32>().unwrap_or(0);
        let value = parts[2..].join(",").replace("\"", "");
        cinfos.push(CInfo { id, code, value });
      }
    } else if line.starts_with("TINFO:") {
      let parts: Vec<&str> = line
        .strip_prefix("TINFO:")
        .unwrap_or("")
        .split(',')
        .collect();
      if parts.len() >= 4 {
        let id = parts[0].parse::<u32>().unwrap_or(0);
        let code = parts[1].parse::<u32>().unwrap_or(0);
        let some_id = parts[2].parse::<u32>().unwrap_or(0);
        let value = parts[3..].join(",").replace("\"", "");

        if tinfos.len() <= id as usize {
          tinfos.push(Vec::new());
        }

        tinfos[id as usize].push(TInfo {
          id,
          code,
          some_id,
          value,
        });
      }
    }
  }

  let mut info = DiskInfo {
    title: "Unknown".to_string(),
    titles: Vec::new(),
  };

  for cinfo in cinfos {
    if cinfo.id == 2 {
      info.title = cinfo.value;
    }
  }

  for (i, tinfo_group) in tinfos.iter().enumerate() {
    let title_name = format!("Title {}", i);
    let mut size_bytes = 0u64;
    let mut duration_secs = 0u32;

    for tinfo in tinfo_group {
      match tinfo.code {
        11 => {
          size_bytes = tinfo.value.parse::<u64>().unwrap_or(0);
        }
        9 => {
          // format is HH:MM:SS
          let parts: Vec<&str> = tinfo.value.split(':').collect();
          if parts.len() == 3 {
            let hours = parts[0].parse::<u32>().unwrap_or(0);
            let minutes = parts[1].parse::<u32>().unwrap_or(0);
            let seconds = parts[2].parse::<u32>().unwrap_or(0);
            duration_secs = hours * 3600 + minutes * 60 + seconds;
          }
        }
        _ => {}
      }
    }

    info.titles.push(DiskTitle {
      id: i as u32,
      name: title_name,
      size_bytes,
      duration_secs,
    });
  }

  info.titles.sort_unstable_by_key(|t| t.size_bytes);
  info.titles.reverse();

  Ok(info)
}

pub fn extract_title(
  drive: &str,
  title_id: u32,
  output_path: &str,
  state: Arc<Mutex<AppState>>,
) -> Result<()> {
  let mut child = Command::new("makemkvcon")
    .args([
      "-r",
      "mkv",
      "--progress=/dev/stdout",
      &format!("dev:{}", drive),
      &format!("{}", title_id),
      output_path,
    ])
    .stdout(Stdio::piped())
    .spawn()?;

  let stdout = child.stdout.take().unwrap();
  let reader = BufReader::new(stdout);

  for line in reader.lines() {
    let line = line?;

    if line.starts_with("PRGC:") {
      let parts: Vec<&str> = line
        .strip_prefix("PRGC:")
        .unwrap_or("")
        .split(',')
        .collect();
      if parts.len() < 3 {
        continue;
      }
      let title = parts[2].replace("\"", "");
      let mut state = state.lock().unwrap();
      if let AppState::TitleExtracting {
        title_id,
        total,
        extracted,
        drive,
        disk_info,
        ..
      } = &*state
      {
        *state = AppState::TitleExtracting {
          drive: drive.to_string(),
          title_id: *title_id,
          total: *total,
          extracted: *extracted,
          task: title,
          disk_info: disk_info.clone(),
        };
      }
    }

    if !line.starts_with("PRGV:") {
      continue;
    }
    let parts: Vec<&str> = line
      .strip_prefix("PRGV:")
      .unwrap_or("")
      .split(',')
      .collect();
    if parts.len() < 3 {
      continue;
    }

    let extracted = parts[0].parse::<u32>().unwrap_or(0);
    let total = parts[2].parse::<u32>().unwrap_or(0);
    let mut state = state.lock().unwrap();
    if let AppState::TitleExtracting {
      drive,
      title_id,
      task,
      disk_info,
      ..
    } = &*state
    {
      *state = AppState::TitleExtracting {
        drive: drive.to_string(),
        title_id: *title_id,
        total,
        extracted,
        task: task.to_string(),
        disk_info: disk_info.clone(),
      };
    }
  }

  let status = child.wait()?;
  if !status.success() {
    bail!("makemkvcon failed with status: {}", status);
  }

  let mut state = state.lock().unwrap();
  let disk_info = if let AppState::TitleExtracting { disk_info, .. } = &*state {
    disk_info.clone()
  } else {
    DiskInfo {
      title: "Unknown".to_string(),
      titles: Vec::new(),
    }
  };
  *state = AppState::Done {
    drive: drive.to_string(),
    title_id,
    disk_info,
    selected: ListState::default().with_selected(Some(0)),
  };

  Ok(())
}
