//
// Sysrepo-examples.
//   rpc_subscribe
//

#[path = "../example_utils.rs"]
mod utils;

use std::env;
use std::thread;
use std::time;

use sysrepo::*;
use yang::data::DataTree;

use utils::*;

/// Show help.
fn print_help(program: &str) {
    println!("Usage: {} <path-to-rpc>", program);
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

    if args.len() != 2 {
        print_help(&program);
        return false;
    }

    let path = args[1].clone();

    println!(r#"Application will subscribe "{}" RPC."#, path);

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
             _sub_id: u32,
             path: &str,
             input: &DataTree,
             _event: Event,
             _request_id: u32,
             output: &mut DataTree|
     -> Result<()> {
        println!(
            "\n\n ========== RPC \"{}\" RECEIVED: =======================\n",
            path
        );
        if let Some(input_node) = input.reference() {
            for node in input_node.traverse() {
                print_node(node);
            }
        }

        if path == "/examples:oper" {
            // TODO: map libyang error into sysrepo error
            output
                .new_path("/examples:oper/ret", Some("-123456"), true)
                .unwrap();
        }

        Ok(())
    };

    // Subscribe for the RPC.
    if let Err(_) = sess.rpc_subscribe(&path, f, 0, 0) {
        return false;
    }

    println!("\n\n ========== LISTENING FOR NOTIFICATIONS ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    true
}
