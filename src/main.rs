use druid::{
    widget::{prelude::*, Button, Flex, Label, LineBreaking, TextBox},
    AppDelegate, AppLauncher, Data, FileDialogOptions, Handled, Lens, LocalizedString, Menu,
    MenuItem, UnitPoint, WidgetExt, WindowDesc, WindowId,
};
use std::{path::Path, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::worker::MsgIn;

mod commands;
mod worker;

#[derive(Clone, Data, Lens, Default)]
struct State {
    database_file: Option<Arc<Path>>,
    sql: String,
}

pub fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title("Hello World!")
        .menu(app_menu)
        .window_size((400.0, 400.0));

    let app = AppLauncher::with_window(main_window).log_to_console();

    // create the initial app state
    let initial_state = State::default();

    let tx = worker::start(app.get_external_handle());
    let delegate = Delegate::new(tx);

    // start the application. Here we pass in the application state.
    app.delegate(delegate)
        .launch(initial_state)
        .expect("Failed to launch application");
}

struct Delegate {
    tx: UnboundedSender<MsgIn>,
}

impl Delegate {
    fn new(tx: UnboundedSender<MsgIn>) -> Self {
        Self { tx }
    }
}

impl AppDelegate<State> for Delegate {
    fn command(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut State,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(druid::commands::OPEN_FILE) {
            data.database_file = Some(file_info.path().into());
            self.tx
                .send(MsgIn::OpenDatabase {
                    db_path: file_info.path().into(),
                })
                .unwrap();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn app_menu<T: Data>(_id: Option<WindowId>, _data: &T, _env: &Env) -> Menu<T> {
    Menu::new("Menu").entry(
        Menu::new("File").entry(
            MenuItem::new("Open database")
                .command(druid::commands::SHOW_OPEN_PANEL.with(FileDialogOptions::default())),
        ),
    )
}

fn build_root_widget() -> impl Widget<State> {
    let buttons = Flex::row().with_child(Button::new("Run query")).padding(4.);

    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &State, _env: &Env| match &data.database_file {
        Some(path) => format!("db file: {}", path.display()),
        None => format!("no db open"),
    })
    .with_line_break_mode(LineBreaking::WordWrap)
    .with_text_size(15.0)
    .expand_height();

    let textbox = TextBox::multiline()
        .with_text_size(15.0)
        .expand_height()
        .lens(State::sql);

    // arrange the two widgets vertically, with some padding
    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Fill)
        .with_child(buttons)
        .with_flex_child(textbox, 1.)
        .with_flex_child(label, 1.)
}
