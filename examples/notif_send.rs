/// An example of an application that sends a notification.
///
/// Adapted from `sysrepo` example `notif_send_example.c`.
use std::env;

use sysrepo::*;
use yang::data::DataTree;

fn main() -> std::result::Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 4 || args.len() == 3 {
        println!(
            "Usage: {} <notification-path> [<node-to-set> <node-value>]",
            args[0]
        );
        return Err(());
    }

    let path = args[1].clone();
    let node_path_val = if args.len() == 4 {
        Some((args[2].clone(), args[3].clone()))
    } else {
        None
    };

    println!("Application will send notification \"{}\".\n", path);

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;
    let ctx = connection.get_context().unwrap();

    // Start session.
    let mut session = connection
        .start_session(Datastore::Running)
        .map_err(|_| ())?;

    // Create the notification.
    let mut notif = DataTree::new(&ctx);
    if let Err(_) = notif.new_path(&path, None, false) {
        println!("Creating notification \"{}\" failed.", path);
        return Err(());
    }

    // Add the input value.
    if let Some((path, value)) = node_path_val {
        if let Err(_) = notif.new_path(&path, Some(&value), false) {
            println!("Creating value \"{}\" failed.", path);
            return Err(());
        }
    }

    // Send the notification.
    if let Err(_) = session.notif_send(&notif, None) {
        println!("Failed to send the notification.");
        return Err(());
    }

    Ok(())
}
