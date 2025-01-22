/// An example of an application providing some operational data by a callback.
///
/// Adapted from `sysrepo` example `oper_data_pull_example.c`.

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

    if args.len() != 3 {
        println!(
            "Usage: {} <module-to-provide-data-from> <path-to-provide>",
            args[0]
        );
        return Err(());
    }

    let mod_name = args[1].clone();
    let path = args[2].clone();

    println!(
        "Application will provide data \"{}\" of \"{}\".",
        path, mod_name
    );

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;

    // Callback
    let dp_get_items_cb = |_sess: &Session,
                           _sub_id: u32,
                           mod_name: &str,
                           path: &str,
                           _request_xpath: Option<&str>,
                           _request_id: u32,
                           output: &mut DataTree<'_>| {
        println!(
            "\n\n ========== DATA FOR \"{}\" \"{}\" REQUESED =======================\n",
            mod_name, path,
        );

        if mod_name == "examples" && path == "/examples:stats" {
            output
                .new_path("/examples:stats/counter", Some("852"), false)
                .unwrap();
            output
                .new_path("/examples:stats/counter2", Some("1052"), false)
                .unwrap();
        }

        Ok(())
    };

    // Start session.
    let session = connection
        .start_session(Datastore::Running)
        .map_err(|_| ())?;

    // Subscribe for the providing the operational data.
    session
        .oper_get_subscribe(&mod_name, &path, dp_get_items_cb, Default::default())
        .map_err(|_| ())?;

    println!("\n\n ========== LISTENING FOR REQUESTS ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    Ok(())
}
