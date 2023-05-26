//! Entry point for the virtual appliance information display
//!
mod ipc;

use std::{net::SocketAddr, sync::mpsc::Receiver};

use clap::Parser;
use cursive::{
    direction::Orientation,
    event::Key,
    reexports::crossbeam_channel::Sender,
    view::{Nameable, Scrollable},
    views::{Dialog, HideableView, LinearLayout, NamedView, Panel, ProgressBar, TextView},
    Cursive,
};
use ipc::IPCEvent;
use vt::{Console, VtNumber};

static COMMON_TITLE: &str = "ArchiveTeam Warrior 4th Edition";
static INFO_TEXT_VIEW: &str = "info_text_view";
static INFO_PROGRESS_BAR: &str = "info_progress_bar";
static INFO_PROGRESS_BAR_HIDEABLE: &str = "info_progress_bar_hideable";

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

    add_info(&mut cursive);
    add_help(&mut cursive);
    add_info_panel(&mut cursive);
    set_up_ipc(&mut cursive, args.ipc_address);

    cursive.add_global_callback(Key::Esc, |c| c.select_menubar());
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
    ipc_receiver: Receiver<IPCEvent>,
    cursive_sender: CursiveSender,
) -> anyhow::Result<()> {
    loop {
        let ipc_event = ipc_receiver.recv()?;

        handle_ipc_event(ipc_event, cursive_sender.clone())?;
    }
}

fn handle_ipc_event(ipc_event: IPCEvent, cursive_sender: CursiveSender) -> anyhow::Result<()> {
    match ipc_event {
        IPCEvent::Progress { text, percent } => {
            cursive_sender
                .send(Box::new(move |cursive| {
                    show_progress(cursive, text, percent)
                }))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
        IPCEvent::Ready { text } => {
            cursive_sender
                .send(Box::new(|cursive| {
                    show_ready(cursive, text);
                }))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
    }

    Ok(())
}

/// Add the Info menu item
fn add_info(cursive: &mut Cursive) {
    cursive.menubar().add_leaf("Info", |c| {
        c.add_layer(
            Dialog::around(TextView::new(generate_info_text()).scrollable())
                .title("Advanced Information")
                .dismiss_button("Close"),
        )
    });
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

    let mut layout = LinearLayout::new(Orientation::Vertical);
    layout.add_child(text_view);
    layout.add_child(HideableView::new(progress_bar).with_name(INFO_PROGRESS_BAR_HIDEABLE));

    cursive.add_layer(Panel::new(layout).title(COMMON_TITLE));
}

/// Update the message displayed to the given text
fn show_progress(cursive: &mut Cursive, text: String, percent: u8) {
    cursive.call_on_name(INFO_TEXT_VIEW, |view: &mut TextView| {
        view.set_content(text);
    });

    cursive.call_on_name(
        INFO_PROGRESS_BAR_HIDEABLE,
        |view: &mut HideableView<NamedView<ProgressBar>>| {
            if percent > 0 {
                view.unhide();
            } else {
                view.hide();
            }
        },
    );

    cursive.call_on_name(INFO_PROGRESS_BAR, |view: &mut ProgressBar| {
        view.set_value(percent.into());
    });
}

/// Show the message that tells the user the web interface is ready to log in
fn show_ready(cursive: &mut Cursive, text: String) {
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

/// Returns the output text for the Info menu item
fn generate_info_text() -> String {
    let programs = vec![
        vec!["ip", "addr", "show"],
        vec!["rc-status"],
        vec!["docker", "ps", "-a"],
    ];

    let mut text = String::new();

    for args in programs {
        text.push_str(&args.join(" "));
        text.push_str("\n\n");

        let (name, args) = args.split_first().unwrap();
        let mut command = std::process::Command::new(name);

        for arg in args {
            command.arg(arg);
        }

        match command.output() {
            Ok(output) => text.push_str(&String::from_utf8_lossy(&output.stdout)),
            Err(error) => text.push_str(&format!("{:#}", error)),
        }

        text.push_str("\n\n");
    }

    text
}
