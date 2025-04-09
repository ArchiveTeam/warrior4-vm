//! Entry point for the virtual appliance information display
//!
mod api;
mod ipc;

use std::{net::SocketAddr, path::Path, sync::mpsc::Receiver};

use api::Request;
use clap::Parser;
use cursive::{
    direction::Orientation,
    event::Key,
    menu::Tree,
    reexports::crossbeam_channel::Sender,
    theme::{Effect, Style},
    utils::markup::StyledString,
    view::{Nameable, Scrollable},
    views::{
        Dialog, HideableView, LayerPosition, LinearLayout, NamedView, Panel, ProgressBar, TextView,
    },
    Cursive,
};
use vt::{Console, VtNumber};

static COMMON_TITLE: &str = "ArchiveTeam Warrior 4th Edition";
static INFO_TEXT_PANEL: &str = "info_text_panel";
static INFO_TEXT_VIEW: &str = "info_text_view";
static INFO_PROGRESS_BAR: &str = "info_progress_bar";
static INFO_PROGRESS_BAR_HIDEABLE: &str = "info_progress_bar_hideable";
static COMMAND_OUTPUT_TEXT_VIEW: &str = "command_output_text_view";

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Bind address for IPC communication
    #[arg(short, long, default_value = "127.0.0.1:40100")]
    ipc_address: SocketAddr,

    /// Switch to and run on the virtual terminal (1 for tty1)
    #[arg(short, long)]
    vt: Option<u8>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(id) = args.vt {
        open_vt(id)?;
    }

    let mut cursive = cursive::default();

    add_status_menu(&mut cursive);
    add_logs_menu(&mut cursive);
    add_help(&mut cursive);
    add_info_panel(&mut cursive);
    set_up_ipc(&mut cursive, args.ipc_address);

    cursive.add_global_callback(Key::Esc, |c| {
        if is_current_info_layer(c) {
            c.select_menubar();
        }
    });
    cursive.run();

    Ok(())
}

fn open_vt(id: u8) -> anyhow::Result<()> {
    let console = Console::open()?;
    let id = VtNumber::new(id as i32);
    console.open_vt(id)?;
    console.switch_to(id)?;

    Ok(())
}

/// Set up threads to process IPC events and update the UI
fn set_up_ipc(cursive: &mut Cursive, address: SocketAddr) {
    let (ipc_sender, ipc_receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || match ipc::run(ipc_sender, address) {
        Ok(_) => {}
        Err(error) => eprintln!("{}", error),
    });

    let cursive_cb = cursive.cb_sink().clone();
    std::thread::spawn(|| match process_callbacks(ipc_receiver, cursive_cb) {
        Ok(_) => {}
        Err(error) => eprintln!("{}", error),
    });
}

type CursiveSender = Sender<Box<dyn FnOnce(&mut Cursive) + Send>>;

fn process_callbacks(
    ipc_receiver: Receiver<Request>,
    cursive_sender: CursiveSender,
) -> anyhow::Result<()> {
    loop {
        let ipc_event = ipc_receiver.recv()?;

        handle_ipc_event(ipc_event, cursive_sender.clone())?;
    }
}

fn handle_ipc_event(ipc_event: Request, cursive_sender: CursiveSender) -> anyhow::Result<()> {
    match ipc_event {
        Request::ProgressInfo { text, percent } => {
            cursive_sender
                .send(Box::new(move |cursive| {
                    show_progress(cursive, text, percent)
                }))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
        Request::ReadyInfo { text }
        | Request::Info { text }
        | Request::Warning { text }
        | Request::Error { text } => {
            cursive_sender
                .send(Box::new(|cursive| {
                    show_message(cursive, text);
                }))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
        Request::CommandOutput { text } => {
            cursive_sender
                .send(Box::new(|cursive| {
                    show_command_output(cursive, text);
                }))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
    }

    Ok(())
}

/// Add the Status menu item
fn add_status_menu(cursive: &mut Cursive) {
    cursive.menubar().add_subtree(
        "Status",
        Tree::new()
            .leaf("IP address", |c| {
                show_command_dialog(&["ip", "addr", "show"], c);
            })
            .leaf("Services", |c| {
                show_command_dialog(&["rc-status", "--all"], c);
            })
            .leaf("Docker", |c| {
                show_command_dialog(&["docker", "ps", "-a"], c);
            })
            .leaf("Filesystem usage", |c| {
                show_command_dialog(&["df", "-h"], c);
            }),
    );
}

/// Add the Logs menu item
fn add_logs_menu(cursive: &mut Cursive) {
    cursive.menubar().add_subtree(
        "Logs",
        Tree::new()
            .leaf("System", |c| {
                show_log_dialog(Path::new("/var/log/messages"), c);
            })
            .leaf("Docker", |c| {
                show_log_dialog(Path::new("/var/log/docker.log"), c);
            })
            .leaf("Warrior appliance", |c| {
                show_log_dialog(Path::new("/var/log/warrior4-appliance.log"), c);
            }),
    );
}

/// Add the help menu item
fn add_help(cursive: &mut Cursive) {
    static HELP_TEXT: &str = "Tip: If can't get out of the virtual machine, press the Host Key (right Ctrl key) to toggle keyboard capture.";

    cursive.menubar().add_leaf("Help", |c| {
        c.add_layer(
            Dialog::around(TextView::new(HELP_TEXT).scrollable())
                .title("Help")
                .dismiss_button("Close"),
        )
    });
}

/// Add the window that shows the status information
fn add_info_panel(cursive: &mut Cursive) {
    let text_view = TextView::new("Waiting for status update...")
        .with_name(INFO_TEXT_VIEW)
        .scrollable();
    let progress_bar = ProgressBar::new().with_name(INFO_PROGRESS_BAR);
    let command_output = TextView::empty()
        .with_name(COMMAND_OUTPUT_TEXT_VIEW)
        .scrollable();

    let mut layout = LinearLayout::new(Orientation::Vertical);
    layout.add_child(text_view);
    layout.add_child(command_output);
    layout.add_child(HideableView::new(progress_bar).with_name(INFO_PROGRESS_BAR_HIDEABLE));

    cursive.add_layer(
        Panel::new(layout)
            .title(COMMON_TITLE)
            .with_name(INFO_TEXT_PANEL),
    );
}

/// Update the message displayed to the given text
fn show_message(cursive: &mut Cursive, text: String) {
    cursive.call_on_name(INFO_TEXT_VIEW, |view: &mut TextView| {
        view.set_content(text);
    });

    cursive.call_on_name(
        INFO_PROGRESS_BAR_HIDEABLE,
        |view: &mut HideableView<NamedView<ProgressBar>>| {
            view.hide();
        },
    );
}

/// Update the message displayed to the given text and progress bar value
fn show_progress(cursive: &mut Cursive, text: String, percent: u8) {
    cursive.call_on_name(INFO_TEXT_VIEW, |view: &mut TextView| {
        view.set_content(text);
    });

    cursive.call_on_name(
        INFO_PROGRESS_BAR_HIDEABLE,
        |view: &mut HideableView<NamedView<ProgressBar>>| {
            view.unhide();
        },
    );

    cursive.call_on_name(INFO_PROGRESS_BAR, |view: &mut ProgressBar| {
        view.set_value(percent.into());
    });
}

/// Update the command output displayed to the given text
fn show_command_output(cursive: &mut Cursive, text: String) {
    cursive.call_on_name(COMMAND_OUTPUT_TEXT_VIEW, |view: &mut TextView| {
        if text.is_empty() {
            view.set_content("");
        } else {
            let content = StyledString::styled(
                format!("\n{}", &text),
                Style::inherit_parent().combine(Effect::Dim),
            );
            view.set_content(content);
        }
    });
}

/// Returns whether the top-most layer is the info text panel
fn is_current_info_layer(cursive: &mut Cursive) -> bool {
    if let Some(view) = cursive.screen().get(LayerPosition::FromFront(0)) {
        if let Some(view) = view.downcast_ref::<NamedView<Panel<LinearLayout>>>() {
            view.name() == INFO_TEXT_PANEL
        } else {
            false
        }
    } else {
        false
    }
}

/// Shows a dialog window containing the contents of a file
fn show_log_dialog(path: &Path, cursive: &mut Cursive) {
    let title = path.to_string_lossy();
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| e.to_string());

    cursive.add_layer(
        Dialog::around(TextView::new(content).no_wrap().scrollable().scroll_x(true))
            .title(title)
            .dismiss_button("Close"),
    );
}

/// Shows a dialog window containing the output of a command
fn show_command_dialog(args: &[&str], cursive: &mut Cursive) {
    let title = args.join(" ");

    let (name, args) = args.split_first().unwrap();
    let mut command = std::process::Command::new(name);

    for arg in args {
        command.arg(arg);
    }

    let content = match command.output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(error) => error.to_string(),
    };

    cursive.add_layer(
        Dialog::around(TextView::new(content).no_wrap().scrollable().scroll_x(true))
            .title(title)
            .dismiss_button("Close"),
    );
}
