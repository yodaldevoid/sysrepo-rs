//
// Sysrepo-examples.
//   application_changes
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
        "Usage: {} <module-to-subscribe> [<xpath-to-subscribe>]",
        program
    );
}

fn print_change(node: &DataTree, oper: ChangeOperation) {
    let node = node.reference().unwrap();
    match oper {
        ChangeOperation::Created
        | ChangeOperation::CreatedLeafListUserOrdered { .. }
        | ChangeOperation::CreatedListUserOrdered { .. } => {
            print!("CREATED: ");
            print_node(node);
        }
        ChangeOperation::Deleted => {
            print!("DELETED: ");
            print_node(node);
        }
        ChangeOperation::Modified {
            previous_value,
            previous_default,
        } => {
            let default = if previous_default { " [default]" } else { "" };
            print!("MODIFIED: {}{} to ", previous_value, default);
            print_node(node);
        }
        ChangeOperation::MovedLeafListUserOrdered {
            previous_value: previous,
        }
        | ChangeOperation::MovedListUserOrdered {
            previous_key: previous,
        } => {
            print!("MOVED: ");
            print_node(node);
            if previous.is_empty() {
                println!(" to the beginning");
            } else {
                println!(" after {}", previous);
            }
        }
    }
}

fn print_current_config(sess: &Session, mod_name: &str) -> Result<()> {
    let xpath = format!("/{}:*//.", mod_name);

    let data = sess.get_data(&xpath, None, Default::default(), Default::default())?;
    for node in data.tree().traverse() {
        print_node(node);
    }

    Ok(())
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
        std::process::exit(1);
    }

    let mod_name = args[1].clone();

    println!(
        r#"Application will watch for changes in "{}"."#,
        if args.len() == 3 {
            args[2].clone()
        } else {
            args[1].clone()
        }
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

    // Read current config.
    println!("");
    println!(" ========== READING RUNNING CONFIG: ==========");
    println!("");
    print_current_config(&sess, &mod_name).unwrap();

    let f = |sess: &Session,
             sub_id: u32,
             mod_name: &str,
             xpath: Option<&str>,
             event: Event,
             _request_id: u32|
     -> Result<()> {
        println!("");
        println!("");
        println!(
            " ========== EVENT ({}) {} CHANGES: ====================================",
            sub_id, event
        );
        println!("");

        let path = if let Some(xpath) = xpath {
            format!("{}//.", xpath)
        } else {
            format!("/{}:*//.", mod_name)
        };
        let changes = match sess.get_changes_iter(&path) {
            Ok(iter) => iter,
            Err(_) => return Ok(()),
        };

        for change in &changes {
            match change {
                Ok((node, oper)) => print_change(&node, oper),
                Err(_) => return Ok(()),
            }
        }

        println!("");
        print!(" ========== END OF CHANGES =======================================");

        if event == Event::Done {
            println!("");
            println!("");
            println!(" ========== CONFIG HAS CHANGED, CURRENT RUNNING CONFIG: ==========");
            println!("");
            print_current_config(sess, mod_name)?;
        }

        Ok(())
    };

    // Subscribe for changes in running config.
    if args.len() == 3 {
        let xpath = args[2].clone();
        match sess.module_change_subscribe(&mod_name, Some(&xpath[..]), f, 0, 0) {
            Err(_) => return false,
            Ok(subscr) => subscr,
        }
    } else {
        match sess.module_change_subscribe(&mod_name, None, f, 0, 0) {
            Err(_) => return false,
            Ok(subscr) => subscr,
        }
    };

    println!("\n\n ========== LISTENING FOR CHANGES ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    true
}
