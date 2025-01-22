/// An example of an application subscribing to a notification.
///
/// Adapted from `sysrepo` example `notif_subscribe_example.c`.

#[path = "../example_utils.rs"]
mod utils;

use std::env;
use std::thread;
use std::time;

use sysrepo::*;
use yang::data::DataTree;

use utils::*;

fn main() -> std::result::Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 && args.len() != 3 {
        println!(
            "Usage: {} <module-with-notification> [<xpath-filtering-notifications>]",
            args[0],
        );
        return Err(());
    }

    let mod_name = args[1].clone();
    let xpath = args.get(2);

    println!(
        "Application will subscribe \"{}\" notifications.\n",
        mod_name
    );

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;

    // Start session.
    let session = connection
        .start_session(Datastore::Running)
        .map_err(|_| ())?;

    // Callback function.
    let notif_cb = |_sess: &Session,
                    _sub_id: u32,
                    _notif_type: NotificationType,
                    tree: &DataTree,
                    _timestamp: time::SystemTime| {
        let node = tree.reference().unwrap();
        println!(
            "\n\n ========== NOTIFICATION \"{}\" RECEIVED =======================\n",
            node.path(),
        );

        for node in node.traverse() {
            print_node(node);
        }
    };

    // Subscribe for the notifications.
    session
        .notif_subscribe(
            &mod_name,
            xpath.map(String::as_str),
            None,
            None,
            notif_cb,
            Default::default(),
        )
        .map_err(|_| ())?;

    println!("\n\n ========== LISTENING FOR NOTIFICATIONS ==========\n");

    // Loop until ctrl-c is pressed / SIGINT is received.
    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    Ok(())
}
