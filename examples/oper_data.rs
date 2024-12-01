//
// Sysrepo-examples.
//   oper_data
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
    println!(
        "Usage: {} <module-to-provide-data-from> <path-to-provide>",
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

    if args.len() != 3 {
        print_help(&program);
        std::process::exit(1);
    }

    let mod_name = args[1].clone();
    let path = args[2].clone();

    println!(
        r#"Application will provide data "{}" of "{}"."#,
        path, mod_name
    );

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let mut sr = match SrConn::new(0) {
        Ok(sr) => sr,
        Err(_) => return false,
    };

    // Callback
    let f = |
        tree: &mut DataTree<'_>,
        sub_id: u32,
        mod_name: &str,
        path: &str,
        _request_xpath: Option<&str>,
        _request_id: u32,
    | {
        println!("");
        println!("");
        println!(
            r#" ========== DATA ({}) FOR "{}" "{}" REQUESED ======================="#,
            sub_id, mod_name, path
        );
        println!("");

        if mod_name == "examples" && path == "/examples:stats" {
            tree.new_path("/examples:stats/counter", Some("852"), false).unwrap();
            tree.new_path("/examples:stats/counter2", Some("1052"), false).unwrap();
        }
    };

    // Start session.
    let sess = match sr.start_session(SrDatastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Subscribe for the providing the operational data.
    if let Err(_) = sess.oper_get_subscribe(&mod_name, &path, f, 0) {
        return false;
    }

    println!("\n\n ========== LISTENING FOR REQUESTS ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    true
}
