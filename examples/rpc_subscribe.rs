/// An example of an application subscribing to an RPC.
///
/// Adapted from `sysrepo` example `rpc_subscribe_example.c`.

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

    if args.len() != 2 {
        println!("Usage: {} <path-to-rpc>", args[0]);
        return Err(());
    }

    let path = args[1].clone();

    println!("Application will subscribe \"{}\" RPC.", path);

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;

    // Start session.
    let session = connection
        .start_session(Datastore::Running)
        .map_err(|_| ())?;

    // Callback function.
    let f = |_sess: &Session,
             _sub_id: u32,
             path: &str,
             input: &DataTree,
             _event: Event,
             _request_id: u32,
             output: &mut DataTree| {
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
    session
        .new_rpc_subscription(&path, f, 0, Default::default())
        .map_err(|_| ())?;

    println!("\n\n ========== LISTENING FOR NOTIFICATIONS ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    Ok(())
}
