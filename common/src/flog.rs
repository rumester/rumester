use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;

use hotwatch::notify::Event;
use hotwatch::EventKind;
use hotwatch::Hotwatch;

use crate::app_data::get_local_appdata_dir;

pub fn begin_flog_watch(app_name: &String) -> Hotwatch {
    let dir = get_log_dir(app_name).unwrap();
    println!("Watching {}", dir.to_str().unwrap());
    let mut watch = Hotwatch::new().unwrap();
    let mut log_file_dir: Option<PathBuf> = None;
    let mut file: Option<fs::File> = None;
    let mut pos: u64 = 0;
    watch
        .watch(dir, move |event: Event| {
            match event.kind {
                EventKind::Create(_) => {
                    println!("Log file created");
                    if log_file_dir.is_some() {
                        return;
                    }

                    log_file_dir = Some(event.paths.get(0).unwrap().to_path_buf());
                    let log_file_dir = log_file_dir.as_ref().unwrap();
                    file = Some(fs::File::open(log_file_dir).unwrap());
                    let file = file.as_ref().unwrap();
                    let data = fs::read_to_string(log_file_dir).unwrap();
                    pos = file.metadata().unwrap().len();
                    print!("{data}");
                }
                EventKind::Modify(_) => {
                    if let Some(file) = &mut file {
                        file.seek(SeekFrom::Start(pos)).unwrap();

                        pos = file.metadata().unwrap().len();

                        let mut new_contents = String::new();
                        file.read_to_string(&mut new_contents).unwrap();

                        print!("{new_contents}");
                    }
                }
                _ => {}
            }
            if let EventKind::Create(_) = event.kind {
                // OK, great, now its time to tail it.
            }
        })
        .expect("Failed to watch log dir");
    watch
}

pub fn get_log_dir(app_name: &String) -> Result<PathBuf, String> {
    let log_dir = get_local_appdata_dir(app_name).join("Roblox/logs");
    Ok(log_dir)
}
