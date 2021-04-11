use druid::{
    widget::{prelude::*, Button, Either, Flex, Label, LineBreaking, RawLabel, TextBox},
    AppDelegate, AppLauncher, ArcStr, Data, FileDialogOptions, Handled, Lens, LocalizedString,
    Menu, MenuItem, UnitPoint, WidgetExt, WindowDesc, WindowId,
};
use std::{path::Path, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

pub mod commands;
mod state;

use self::state::{App, State, StateOpen};
use crate::db;

pub fn run() {
    // describe the main window
    let main_window = WindowDesc::new(main_window())
        .title("Hello World!")
        .menu(app_menu)
        .window_size((400.0, 400.0));

    let app = AppLauncher::with_window(main_window).log_to_console();

    let tx = db::start(app.get_external_handle());

    // start the application. Here we pass in the application state.
    let mut data = App::default();
    data.log("Starting application");
    app.delegate(Delegate::new(tx))
        .launch(data)
        .expect("Failed to launch application");
}

struct Delegate {
    tx: UnboundedSender<db::MsgIn>,
    logs_window: Option<WindowId>,
}

impl Delegate {
    fn new(tx: UnboundedSender<db::MsgIn>) -> Self {
        Self {
            tx,
            logs_window: None,
        }
    }

    fn send(&self, msg: db::MsgIn) {
        self.tx.send(msg).unwrap()
    }
}

impl AppDelegate<App> for Delegate {
    fn command(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        target: druid::Target,
        cmd: &druid::Command,
        data: &mut App,
        _env: &Env,
    ) -> Handled {
        if cmd.get(commands::NEW_DATABASE).is_some() {
            data.request_open_memory_db();
            self.send(db::MsgIn::OpenDatabase { db_path: None });
            Handled::Yes
        } else if let Some(file_info) = cmd.get(druid::commands::OPEN_FILE) {
            data.request_open_db(file_info.path().into());
            self.send(db::MsgIn::OpenDatabase {
                db_path: Some(file_info.path().into()),
            });
            Handled::Yes
        } else if cmd.get(commands::DATABASE_OPENED).is_some() {
            data.db_opened();
            Handled::Yes
        } else if let Some(e) = cmd.get(commands::DATABASE_OPEN_FAILED) {
            data.db_open_failed(e);
            Handled::Yes
        } else if cmd.get(commands::CLOSE_DATABASE).is_some() {
            data.close_db();
            self.send(db::MsgIn::CloseDatabase);
            Handled::Yes
        } else if cmd.get(commands::DATABASE_CLOSED).is_some() {
            data.db_closed();
            Handled::Yes
        } else if let Some(sql) = cmd.get(commands::START_QUERY) {
            data.start_query();
            self.send(db::MsgIn::RunQuery {
                sql: (**sql).clone(),
            });
            Handled::Yes
        } else if cmd.get(commands::SHOW_LOG).is_some() {
            if self.logs_window.is_none() {
                let desc = WindowDesc::new(log_window())
                    .title("Database Logs")
                    .window_size((400.0, 400.0));
                let id = desc.id;
                ctx.new_window(desc);
                self.logs_window = Some(id);
            }
            Handled::Yes
        } else if cmd.get(druid::commands::CLOSE_WINDOW).is_some() {
            if let druid::Target::Window(window_id) = target {
                match &self.logs_window {
                    Some(id) if *id == window_id => {
                        self.logs_window = None;
                    }
                    _ => (),
                }
            }
            Handled::No
        } else {
            Handled::No
        }
    }
}

fn app_menu(_id: Option<WindowId>, _data: &App, _env: &Env) -> Menu<App> {
    Menu::new("Menu")
        .entry(
            Menu::new("File")
                .entry(MenuItem::new("New").command(commands::NEW_DATABASE))
                .entry(
                    MenuItem::new("Open").command(
                        druid::commands::SHOW_OPEN_PANEL.with(FileDialogOptions::default()),
                    ),
                )
                .entry(
                    MenuItem::new("Save As")
                        .command(
                            druid::commands::SHOW_OPEN_PANEL.with(FileDialogOptions::default()),
                        )
                        .enabled_if(|data: &App, _| matches!(data.state, State::Open(_))),
                )
                .entry(
                    MenuItem::new("Close")
                        .command(commands::CLOSE_DATABASE)
                        .enabled_if(|data: &App, _| matches!(data.state, State::Open(_))),
                )
                .separator()
                .entry(MenuItem::new("Quit").command(druid::commands::QUIT_APP)),
        )
        .entry(Menu::new("View").entry(MenuItem::new("Log").command(commands::SHOW_LOG)))
}

fn main_window() -> impl Widget<App> {
    State::matcher()
        .closed(Label::new("no database open").center())
        .closing(Label::new("database closing").center())
        .opening(Label::new("database opening").center())
        .open(db_open())
        .lens(App::state)
}

fn log_window() -> impl Widget<App> {
    RawLabel::new().expand().lens(App::logs)
}

fn db_open() -> impl Widget<StateOpen> {
    let buttons = Either::new(
        |data: &StateOpen, _| data.query_running,
        Flex::row()
            .with_child(
                Button::new("Cancel query").on_click(|ctx, data: &mut StateOpen, _| {
                    ctx.submit_command(commands::CANCEL_QUERY);
                }),
            )
            .padding(4.),
        Flex::row()
            .with_child(
                Button::new("Run query").on_click(|ctx, data: &mut StateOpen, _| {
                    ctx.submit_command(commands::START_QUERY.with(data.sql.clone()));
                }),
            )
            .padding(4.),
    );

    let textbox = TextBox::multiline()
        .with_text_size(15.0)
        .expand_height()
        .lens(StateOpen::sql);

    let label = Label::new("tmp")
        .with_line_break_mode(LineBreaking::WordWrap)
        .with_text_size(15.0)
        .expand_height();

    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Fill)
        .with_child(buttons)
        .with_flex_child(textbox, 1.)
        .with_flex_child(label, 1.)
}
