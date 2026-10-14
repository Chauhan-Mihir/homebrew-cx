//! A deliberately trivial stand-in for the real `cx` binary.
//!
//! It exists only to rehearse the distribution path end to end — archive,
//! release, Homebrew/Scoop manifest, install, upgrade, uninstall — without
//! publishing anything proprietary. It contains no product logic, no
//! network access and no dependencies, so it is safe to put in a public
//! repository.
//!
//! The version is stamped at build time by `build.rs` from `CX_VERSION`,
//! mirroring `crates/build-info` in the toolkit, so the rehearsal
//! exercises the same version-stamping path the real pipeline uses.

const VERSION: &str = env!("CX_BUILD_VERSION");
const BUILD_TIME: &str = env!("CX_BUILD_TIME");
const TARGET: &str = env!("CX_BUILD_TARGET");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        // Bare `--version` output is what the install tests assert on, so
        // it stays on one line and leads with the binary name.
        Some("--version" | "-V" | "version") => println!("cx {VERSION}"),
        Some("--help" | "-h" | "help") | None => help(),
        Some(other) => {
            eprintln!("cx: unknown command '{other}'");
            help();
            std::process::exit(2);
        }
    }
}

fn help() {
    println!("cx {VERSION} ({TARGET}, built {BUILD_TIME})");
    println!();
    println!("Distribution rehearsal stand-in for the Commerce+ eXtensibility Toolkit.");
    println!("It ships no product functionality.");
    println!();
    println!("Usage: cx <command>");
    println!();
    println!("Commands:");
    println!("  --version    Print the version and exit");
    println!("  --help       Print this message and exit");
}
