//
// Sysrepo-examples.
//   notif_send
//

use std::env;

use sysrepo::*;
use yang::data::DataTree;

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <notification-path> [<node-to-set> <node-value>]",
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

    if args.len() < 2 || args.len() > 4 || args.len() == 3 {
        print_help(&program);
        return false;
    }

    let path = args[1].clone();
    let node_path_val = if args.len() == 4 {
        Some((args[2].clone(), args[3].clone()))
    } else {
        None
    };

    println!(
        r#"Application will send notification "{}" notification."#,
        path
    );

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let sr = match Connection::new(Default::default()) {
        Ok(sr) => sr,
        Err(_) => return false,
    };

    // Get Lib Yang Context from sysrepo connection.
    let ly_ctx = sr.get_context().unwrap();

    // Start session.
    let mut sess = match sr.start_session(Datastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Create the notification.
    let mut notif = DataTree::new(&ly_ctx);
    if let Err(_) = notif.new_path(&path, None, false) {
        println!(r#"Creating notification "{}" failed."#, path);
        return false;
    }

    // Add the input value.
    if let Some((path, value)) = node_path_val {
        if let Err(_) = notif.new_path(&path, Some(&value), false) {
            println!(r#"Creating value "{}" failed."#, path);
            return false;
        }
    }

    // Send the notification.
    if let Err(_) = sess.notif_send_tree(&notif, 0, false) {
        return false;
    }

    true
}
