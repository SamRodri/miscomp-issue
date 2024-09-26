mod help;
#[macro_use]
mod util;

use util::*;

fn main() {
    let (arg_cmd, args) = args();
    match arg_cmd.as_str() {
        "run-r" => run_r(args),
        "help" | "--help" | "-h" | "" => help(args),
        _other => cmd("cargo", [arg_cmd].into_iter().chain(args))
            .status()
            .success_or_die("cannot run cargo"),
    }
}

/// do build-r [--bleed] [--dev]
///    RUn miscomp-issue release profile+features
///
///    ARGS
///       --bleed - Build with nightly compiler optimizations.
///       --dev   - Build with dev profile and release features.
fn run_r(args: Vec<String>) {
    let (_, options, args) = split_args(&args, &[], &["--bleed", "--dev"], true, true);
    let bleed = options.contains_key("--bleed");
    let dev = options.contains_key("--dev");

    let mut cmd = std::process::Command::new("cargo");
    if bleed {
        cmd.arg("+nightly");
    }
    cmd.args([
        "run",
        if dev {
            "--profile=dev"
        } else {
            "--profile=release"
        },
        "--no-default-features",
        "--features=release",
        "--package",
        "miscomp-issue",
    ])
    .args(args);

    if bleed {
        // -Zshare-generics               - halves binary size
        // -C link-args=-znostart-stop-gc - Fixes build error
        cmd.env(
            "RUSTFLAGS",
            format!(
                "{} -Z share-generics -C link-args=-znostart-stop-gc",
                std::env::var("RUSTFLAGS").unwrap_or_default()
            ),
        );
    }
    cmd.status().success_or_die("release build failed");
}

/// do help
///    Prints this help
fn help(_: Vec<String>) {
    self::help::print();
}
