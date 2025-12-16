use std::{fs, process::Command};

use color_eyre::eyre::Result;

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

struct CInfo {
  id: u32,
  code: u32,
  value: String,
}

struct TInfo {
  id: u32,
  code: u32,
  some_id: u32,
  value: String,
}

pub struct DiskInfo {
  pub title: String,
  pub titles: Vec<DiskTitle>,
}

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
        let value = parts[2..].join(",");
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
        let value = parts[3..].join(",");

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

  Ok(info)
}
