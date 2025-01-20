//
// Sysrepo-examples.
//   notif_subscribe
//

#[path = "../example_utils.rs"]
mod utils;

use std::env;
use std::thread;
use std::time;

use sysrepo::*;
use yang::data::DataTree;
use yang::ffi::timespec;

use utils::*;

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <module-with-notification> [<xpath-filtering-notifications>]",
        program
    );
}

/// Main.
fn main() {
    if run() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn run() -> bool {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() != 2 && args.len() != 3 {
        print_help(&program);
        return false;
    }

    let mod_name = args[1].clone();
    let xpath = if args.len() == 3 {
        Some(args[2].clone())
    } else {
        None
    };

    println!(
        r#"Application will subscribe "{}" notifications."#,
        mod_name
    );

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let sr = match Connection::new(Default::default()) {
        Ok(sr) => sr,
        Err(_) => return false,
    };

    // Start session.
    let sess = match sr.start_session(Datastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Callback function.
    let f = |_sess: &Session,
             sub_id: u32,
             _notif_type: NotificationType,
             tree: &DataTree,
             _timestamp: *mut timespec| {
        let node = tree.reference().unwrap();
        println!("");
        println!("");
        println!(
            r#" ========== NOTIFICATION ({}) "{}" RECEIVED ======================="#,
            sub_id,
            node.path(),
        );
        println!("");

        for node in node.traverse() {
            print_node(node);
        }
    };

    // Subscribe for the notifications.
    if let Err(_) = sess.notif_subscribe(&mod_name, xpath.as_deref(), None, None, f, 0) {
        return false;
    }

    println!("\n\n ========== LISTENING FOR NOTIFICATIONS ==========\n");

    // Loop until ctrl-c is pressed / SIGINT is received.
    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    true
}
