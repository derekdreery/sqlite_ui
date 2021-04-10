use druid::{ExtEventSink, Target};
use std::{path::PathBuf, thread};

use crate::commands::DATABASE_OPENED;

/// Messages to the worker thread
#[derive(Debug)]
pub enum MsgIn {
    Quit,
    OpenDatabase { db_path: PathBuf },
}

/// Data that lives on the worker thread (in a tokio event loop)
pub enum DbWorker {
    Closed,
}

impl DbWorker {
    fn new() -> Self {
        DbWorker::Closed
    }
}

pub fn start(sink: ExtEventSink) -> tokio::sync::mpsc::UnboundedSender<MsgIn> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                match rx.recv().await {
                    Some(MsgIn::OpenDatabase { db_path }) => {
                        sink.submit_command(DATABASE_OPENED, (), Target::Global)
                            .unwrap();
                    }
                    Some(MsgIn::Quit) | None => break,
                }
            }
        });
    });
    tx
}
