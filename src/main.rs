#![feature(path_ext)]

use std::env;
use std::fs::{self, PathExt};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command};
use std::string::String;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};

mod file_watch;
use file_watch::{FileWatch, WATCH_CREATE, WATCH_MODIFY};

fn std_err(msg: &str) {
    writeln!(&mut io::stderr(), "{}", msg).unwrap();
}

extern crate nix;
use nix::sys::signal;

static RECEIVED_INTERRUPT: AtomicBool = ATOMIC_BOOL_INIT;

extern fn handle_sigint(_:i32) {
  RECEIVED_INTERRUPT.store(true, Ordering::Relaxed);
}

fn setup_sigint_handler() {
    let sig_action = signal::SigAction::new(
            handle_sigint,
            signal::SockFlag::empty(),
            signal::SigSet::empty());
    unsafe {
        let _ = signal::sigaction(signal::SIGINT, &sig_action);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path_str = match extract_path_from(&args) {
        Ok(p) => p,
        Err(msg) => {
            std_err(msg);
            process::exit(1);
        }
    };

    let path = if is_absolute(&path_str) {
        PathBuf::from(&path_str)
    }
    else {
        env::current_dir().unwrap().join(&path_str)
    };

    println!("Watching {} ...", path.display());

    let mut file_watch = FileWatch::new().unwrap();
    {
        let mut add_file_watch = |dir: &Path| {
            file_watch.add_watch(dir, WATCH_CREATE | WATCH_MODIFY).unwrap();
        };

        visit_all_dirs_in(path.as_path(), &mut add_file_watch).unwrap();
    }

    setup_sigint_handler();

    loop {
        if RECEIVED_INTERRUPT.load(Ordering::Relaxed) {
            println!("Shutting down..");
            return;
        }

        let events = match file_watch.wait_for_events() {
            Ok(events) => events,
            Err(_) => continue,
        };
        for event in events.iter() {

            if !event.is_dir() && event.name.ends_with(".rs") {
                // rerun tests
                match run_cargo_test_for(path.as_path()) {
                    Ok(child) => {
                        let mut child = child;
                        let _ = child.wait();
                    },
                    Err(_) => { std_err("Unable to run cargo test."); },
                };
                break;
            }
        }
    }
}

fn visit_all_dirs_in(dir: &Path, visit: &mut FnMut(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        visit(dir);

        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            if entry.path().is_dir() {
                try!(visit_all_dirs_in(&entry.path(), visit));
            }
        }
    }

    Ok(())
}

fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

#[test]
fn test_is_absolute() {
    assert_eq!(false, is_absolute("."));
}

fn run_cargo_test_for(path: &Path) -> io::Result<Child> {
    // pseudo clear console
    println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

    let result = Command::new("cargo")
        .arg("test")
        .current_dir(path)
        .spawn();

    if result.is_err() {
        std_err("Unable to run cargo test.");
    }

    result
}

fn extract_path_from(args: &[String]) -> Result<&str, &'static str> {
    if args.len() < 2 {
        return Err("No path given!");
    }

    Ok( &args[1] )
}

#[test]
fn test_extract_path_from() {
    let input = vec!["name-of-program".to_string(), "/some/simple/path".to_string()];

    let actual = extract_path_from(&input).unwrap();

    assert_eq!(input[1], actual);
}
