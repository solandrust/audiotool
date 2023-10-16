mod config {
    use std::path::PathBuf;

    pub struct Config {
        pub reference_tracks_dir: PathBuf,
        pub reference_track_regex: String,
        pub out_root_dir: PathBuf,
        pub outputs: Vec<OutDesc>,
    }

    pub struct OutDesc {
        pub dir: PathBuf,
        pub format: Format,
    }

    pub enum Format {
        Flac(FlacFormat),
        Alac,
        Vorbis,
        Mp3,
        Aac,
    }

    pub struct FlacFormat {
        pub bit_depth: u32,
        pub sample_rate: u32,
    }
}

pub use config::*;

use rx::prelude::*;
use rx::rayon::{self, prelude::*};

use rx::walkdir::{self, WalkDir, DirEntry};
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::path::{PathBuf, Path};

pub enum Request {
    Cancel,
}

pub enum Response {
    NextResult(AnyResult<ConvertResult>),
    Done,
}

pub struct ConvertResult {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
}

pub fn spawn(config: Config) -> (
    SyncSender<Request>,
    Receiver<Response>,
) {
    let (in_tx, in_rx) = sync_channel(1);
    let (out_tx, out_rx) = sync_channel(1);

    thread::spawn(move || {
        run(config, in_rx, out_tx)
    });

    (in_tx, out_rx)
}

fn run(
    config: Config,
    rx: Receiver<Request>,
    tx: SyncSender<Response>,
) {
    let cancel = Arc::new(AtomicBool::from(false));

    thread::spawn({
        let cancel = cancel.clone();
        move || {
            for req in rx.iter() {
                match req {
                    Request::Cancel => {
                        cancel.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }
    });

    WalkDir::new(&config.reference_tracks_dir)
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| {
            let keep_going = convert_entry(
                &config, entry, &tx, &cancel,
            );

            return keep_going;
        });

    let _ = tx.send(Response::Done);
}

fn convert_entry(
    config: &Config,
    entry: Result<DirEntry, walkdir::Error>,
    tx: &SyncSender<Response>,
    cancel: &AtomicBool,
) -> Option<()> {
    let entry = match entry {
        Err(err) => {
            tx.send(Response::NextResult(Err(err.into())));
            return Some(());
        }
        Ok(entry) => entry,
    };

    let res = convert_entry2(
        config, &entry, cancel,
    );

    match res {
        Ok(Some(res)) => {
            tx.send(Response::NextResult(Ok(res)));
            Some(())
        }
        Ok(None) => {
            None
        }
        Err(err) => {
            tx.send(Response::NextResult(Err(err)));
            Some(())
        }
    }
}

fn convert_entry2(
    config: &Config,
    entry: &DirEntry,
    cancel: &AtomicBool,
) -> AnyResult<Option<ConvertResult>> {
    let in_path = entry.path();
    let relative_path = in_path.strip_prefix(&config.reference_tracks_dir)?;
    let out_path = config.out_root_dir.join(&relative_path);
    todo!()
}

fn convert_file(
    in_file: &Path,
    out_file: &Path,
    out_format: Format,
    cancel: &AtomicBool,
) -> AnyResult<Option<ConvertResult>> {
    todo!();

    if cancel.load(Ordering::SeqCst) {
        return Ok(None);
    }

    todo!();
}

