#![warn(missing_docs)]

//! Code used to drive the creation of documentation for Rust Crates.

extern crate rustdoc;
extern crate clap;

use clap::{App, Arg, SubCommand};

use rustdoc::{build, Config, Result, Verbosity};

use std::io::{Write, stderr};
use std::process;
use std::path::PathBuf;

static ALL_ARTIFACTS: &[&str] = &["frontend", "json"];
static DEFAULT_ARTIFACTS: &[&str] = &["frontend"];

fn run() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    let matches = App::new("rustdoc")
        .version(version)
        .author("Steve Klabnik <steve@steveklabnik.com>")
        .about("Generate web-based documentation from your Rust code.")
        .arg(
            Arg::with_name("manifest-path")
                .long("manifest-path")
                // remove the unwrap in Config::new if this default_value goes away
                .default_value("./Cargo.toml")
                .help("The path to the Cargo manifest of the project you are documenting."),
        )
        .arg(Arg::with_name("quiet").short("q").long("quiet").help(
            "No output printed to stdout",
        ))
        .arg(Arg::with_name("verbose").short("v").long("verbose").help(
            "Use verbose output",
        ))
        .subcommand(
            SubCommand::with_name("build")
                .about("generates documentation")
                .arg(
                    Arg::with_name("artifacts")
                        .long("emit")
                        .use_delimiter(true)
                        .takes_value(true)
                        .possible_values(ALL_ARTIFACTS)
                        .help("Build artifacts to produce. Defaults to just the frontend."),
                )
                .arg(Arg::with_name("open").short("o").long("open").help(
                    "Open the docs in a web browser after building.",
                )),
        )
        .subcommand(SubCommand::with_name("open").about(
            "opens the documentation in a web browser",
        ))
        .subcommand(SubCommand::with_name("test").about(
            "runs documentation tests in the current crate",
        ))
        .get_matches();

    // unwrap is okay because we take a default value
    let manifest_path = PathBuf::from(&matches.value_of("manifest-path").unwrap());
    let verbosity = if matches.is_present("quiet") {
        Verbosity::Quiet
    } else if matches.is_present("verbose") {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };
    let config = Config::new(verbosity, manifest_path)?;

    match matches.subcommand() {
        ("build", Some(matches)) => {
            // FIXME: Workaround for clap #1056. Use `.default_value()` once the issue is fixed.
            let artifacts: Vec<&str> = matches
                .values_of("artifacts")
                .map(|values| values.collect())
                .unwrap_or_else(|| {
                    DEFAULT_ARTIFACTS.iter().map(|&artifact| artifact).collect()
                });
            build(&config, &artifacts)?;
            if matches.is_present("open") {
                config.open_docs()?;
            }
        }
        ("open", _) => {
            // First build the docs if they are not yet built.
            if !config.output_path().exists() {
                build(&config, DEFAULT_ARTIFACTS)?;
            }
            config.open_docs()?;
        }
        ("test", _) => {
            build(&config, ALL_ARTIFACTS)?;
            rustdoc::test(&config)?;
        }
        // default is to build
        _ => build(&config, DEFAULT_ARTIFACTS)?,
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        let stderr = &mut stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "Error: {}", e).expect(errmsg);

        writeln!(stderr, "Caused by: {}", e.cause()).expect(errmsg);

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        writeln!(stderr, "Backtrace, if any: {:?}", e.backtrace()).expect(errmsg);

        process::exit(1);
    }
}
