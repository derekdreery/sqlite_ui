use anyhow::{format_err, Error};
use druid::{ExtEventSink, Selector, Target};
use sqlx::{prelude::*, SqlitePool};
use std::{
    any::Any,
    mem,
    path::{Path, PathBuf},
    thread,
};

use crate::gui::commands::{DATABASE_CLOSED, DATABASE_OPENED, DATABASE_OPEN_FAILED, QUERY_ERROR};

/// Messages to the worker thread
#[derive(Debug)]
pub enum MsgIn {
    Quit,
    /// `None` is an in-memory database.
    OpenDatabase {
        db_path: Option<PathBuf>,
    },
    CloseDatabase,
    RunQuery {
        sql: String,
    },
}

/// Data that lives on the worker thread (in a tokio event loop).
struct Worker {
    state: State,
    sink: ExtEventSink,
}

/// A state machine.
enum State {
    /// Used when we need to move the data out of the state temporarily.
    Taken,
    /// No database open.
    Closed,
    /// `None` is an in-memory database.
    Open { db: SqlitePool },
}

impl Worker {
    fn new(sink: ExtEventSink) -> Self {
        Worker {
            state: State::Closed,
            sink,
        }
    }

    fn send<T: Any + Send>(&self, sel: Selector<T>, payload: impl Into<Box<T>>) {
        self.sink
            .submit_command(sel, payload, Target::Global)
            .unwrap();
    }

    async fn open(&mut self, path: Option<PathBuf>) {
        // TODO check state, and gracefully close existing connection if necessary.
        let connection_str = match build_connection_str(path.as_ref()) {
            Ok(path) => path,
            Err(e) => {
                self.send(DATABASE_OPEN_FAILED, e);
                self.state = State::Closed;
                return;
            }
        };
        let pool = match SqlitePool::connect(&connection_str).await {
            Ok(pool) => pool,
            Err(e) => {
                self.send(DATABASE_OPEN_FAILED, Error::from(e));
                self.state = State::Closed;
                return;
            }
        };
        self.send(DATABASE_OPENED, ());
        self.state = State::Open { db: pool };
    }

    async fn close(&mut self) {
        if let State::Open { db, .. } = &self.state {
            db.close().await;
            self.send(DATABASE_CLOSED, ());
        }
    }

    async fn query(&mut self, sql: String) {
        if let State::Open { db } = &self.state {
            match db.execute(sql.as_str()).await {
                Ok(_) => {}
                Err(e) => self.send(QUERY_ERROR, Error::from(e)),
            }
        } else {
            unreachable!()
        }
    }
}

pub fn start(sink: ExtEventSink) -> tokio::sync::mpsc::UnboundedSender<MsgIn> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    thread::spawn(move || {
        let mut db = Worker::new(sink);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                match rx.recv().await {
                    Some(MsgIn::OpenDatabase { db_path }) => {
                        db.open(db_path.clone()).await;
                    }
                    Some(MsgIn::CloseDatabase) => {
                        db.close().await;
                    }
                    Some(MsgIn::RunQuery { sql }) => {
                        // todo task::spawn so we can cancel
                        db.query(sql).await;
                    }
                    Some(MsgIn::Quit) | None => break,
                }
            }
        });
    });
    tx
}

fn build_connection_str(path: Option<impl AsRef<Path>>) -> Result<String, Error> {
    match path {
        Some(path) => {
            let path = path.as_ref();
            let path = match path.to_str() {
                Some(path) => path,
                None => {
                    return Err(format_err!(
                        "path {} is not utf_8, which is required for sqlite connection",
                        path.display()
                    ))
                }
            };
            Ok(format!("sqlite:file://{}", path))
        }
        None => Ok(format!("sqlite:file::memory:?cache=shared")),
    }
}
