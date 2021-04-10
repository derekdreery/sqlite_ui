use anyhow::Error;
use druid::{ArcStr, Data, Lens, WindowId};
use druid_enums::Matcher;
use std::{fmt::Write as _, path::Path, sync::Arc};

#[derive(Clone, Data, Lens, Default)]
pub struct App {
    pub logs: String,
    pub state: State,
}

impl App {
    pub fn log(&mut self, msg: impl AsRef<str>) {
        writeln!(&mut self.logs, "{}", msg.as_ref()).unwrap()
    }

    pub fn request_open_memory_db(&mut self) {
        self.log("Opening in-memory database");
        self.state = State::Opening(None);
    }

    pub fn request_open_db(&mut self, path: Arc<Path>) {
        self.log(format!("Opening database at \"{}\"", path.display()));
        self.state = State::Opening(Some(path));
    }

    pub fn db_opened(&mut self) {
        self.log("Database successfully opened");
        if let State::Opening(ref path) = self.state {
            self.state = State::Open(StateOpen::new(path.clone()));
        } else {
            unreachable!()
        }
    }

    pub fn db_open_failed(&mut self, e: &Error) {
        let msg = match &self.state {
            State::Opening(Some(path)) => {
                format!(r#"Error opening database at "{}": {}"#, path.display(), e)
            }
            State::Opening(None) => format!(r#"Error opening in-memory database: {}"#, e),
            _ => unreachable!(),
        };
        self.log(msg);
        self.state = State::Closed;
    }

    pub fn close_db(&mut self) {
        let (logs, state) = self.split_borrow();
        let path = match state {
            State::Open(state) => {
                let msg = match &state.path {
                    Some(path) => format!("Closing database at \"{}\"\n", path.display()),
                    None => format!("Closing and discarding in-memory database\n"),
                };
                logs.push_str(&msg);
                state.path.clone()
            }
            _ => unreachable!(),
        };
        *state = State::Closing(path);
    }

    pub fn db_closed(&mut self) {
        let (logs, state) = self.split_borrow();
        match state {
            State::Closing(path) => {
                let msg = match path {
                    Some(path) => format!("Closed database at \"{}\"\n", path.display()),
                    None => format!("Closed in-memory database\n"),
                };
                logs.push_str(&msg);
            }
            _ => unreachable!(),
        };
        *state = State::Closed;
    }

    pub fn start_query(&mut self) {
        let (logs, state) = self.split_borrow();
        if let State::Open(state) = state {
            assert!(!state.query_running);
            logs.push_str(&format!("Running query \"{}\"\n", state.sql));
            state.query_running = true;
        } else {
            unreachable!()
        }
    }

    pub fn cancel_query(&mut self) {
        let (logs, state) = self.split_borrow();
        if let State::Open(state) = state {
            assert!(state.query_running);
            logs.push_str(&format!("Cancelling query\n"));
            state.query_running = true;
        } else {
            unreachable!()
        }
    }

    pub fn query_response(&mut self) {
        todo!()
    }

    fn split_borrow(&mut self) -> (&mut String, &mut State) {
        (&mut self.logs, &mut self.state)
    }
}

#[derive(Clone, Data, Matcher)]
pub enum State {
    Closed,
    Closing(Option<Arc<Path>>),
    Opening(Option<Arc<Path>>),
    Open(StateOpen),
}

impl Default for State {
    fn default() -> Self {
        State::Closed
    }
}

#[derive(Clone, Data, Lens)]
pub struct StateOpen {
    pub query_running: bool,
    pub path: Option<Arc<Path>>,
    pub sql: Arc<String>,
    pub prev_query_result: QueryResult,
}

impl StateOpen {
    fn new(path: Option<Arc<Path>>) -> Self {
        StateOpen {
            query_running: false,
            path,
            sql: Arc::new(String::new()),
            prev_query_result: QueryResult::None,
        }
    }
}

#[derive(Clone, Data, Matcher)]
pub enum QueryResult {
    None,
    // todo
    Some(()),
    Err(ArcStr),
}

impl Default for QueryResult {
    fn default() -> Self {
        QueryResult::None
    }
}
