use anyhow::Error;
use druid::Selector;

pub const NEW_DATABASE: Selector = Selector::new("sqlite_ui.database-new");
pub const DATABASE_OPENED: Selector = Selector::new("sqlite_ui.database-opened");
pub const DATABASE_OPEN_FAILED: Selector<Error> = Selector::new("sqlite_ui.database-open-failed");
pub const CLOSE_DATABASE: Selector = Selector::new("sqlite_ui.database-close");
pub const DATABASE_CLOSED: Selector = Selector::new("sqlite_ui.database-closed");
pub const START_QUERY: Selector<String> = Selector::new("sqlite_ui.database-query-start");
pub const CANCEL_QUERY: Selector = Selector::new("sqlite_ui.database-query-cancel");
pub const QUERY_COMPLETED: Selector = Selector::new("sqlite_ui.database-query-completed");
pub const QUERY_ERROR: Selector<Error> = Selector::new("sqlite_ui.database-query-error");

pub const SHOW_LOG: Selector = Selector::new("sqlite_ui.log-show");
