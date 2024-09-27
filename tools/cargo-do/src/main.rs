mod help;
#[macro_use]
mod util;
mod pack_android;
mod pack_deb;

use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::Stdio,
    time::Duration,
};

use util::*;

fn main() {
    let (arg_cmd, args) = args();
    match arg_cmd.as_str() {
        "fmt" => fmt(args),
        "l10n" => l10n(args),
        "pack" => pack(args),
        "build-r" => build_r(args),
        "build-ndk" => build_ndk(args),
        "run-r" => run_r(args),
        "update" => update(args),
        "test-apk" => test_apk(args),
        "help" | "--help" | "-h" | "" => help(args),
        _other => cmd("cargo", [arg_cmd].into_iter().chain(args))
            .status()
            .success_or_die("cannot run cargo"),
    }
}

/// do fmt
///    Calls cargo zng fmt
fn fmt(args: Vec<String>) {
    cmd("cargo", &["zng", "fmt"])
        .args(args)
        .status()
        .success_or_die("cannot zng fmt")
}

/// do l10n
///    Scraps localization text
fn l10n(args: Vec<String>) {
    cmd(
        "cargo",
        &[
            "zng",
            "l10n",
            "--package",
            "miscomp-issue",
            "--output",
            "res/l10n",
            "--clean",
        ],
    )
    .args(args)
    .status()
    .success_or_die("cannot scrap l10n")
}

/// do pack <PACKAGE> [--no-build]
///    Compile with release profile+features and package
///
///    ARGS
///       <PACKAGE>  - Name of a pack/{PACKAGE}
///       --no-build - Skips release build
fn pack(args: Vec<String>) {
    // parse args
    let (package, options, args) = split_args(&args, &["PACKAGE"], &["--no-build"], false, true);
    let package = package[0].as_str();

    if package == "deb" && options.contains_key("--changelog") {
        return pack_deb::changelog();
    }
    if package == "android" && options.contains_key("--locales") {
        return pack_android::locales();
    }

    if options.contains_key("--no-build") {
        println!("skipping release build");
    } else {
        println!("building release");
        if package == "android" {
            build_ndk(vec!["--release".to_owned()]);
        } else {
            build_r(vec![]);
        }
    }

    // pack
    println!("packing");
    let mut pack_cmd = cmd(
        "cargo",
        &[
            "zng",
            "res",
            "--metadata",
            "crates/miscomp-issue-mobile/Cargo.toml",
            &format!("pack/{package}"),
            &format!("target/pack/{package}"),
            "--pack",
        ],
    );
    pack_cmd.args(&args);

    let name = format!("miscomp-issue{}", std::env::consts::EXE_SUFFIX);

    let app_path = Path::new("target")
        .canonicalize()
        .unwrap()
        .join("release")
        .join(&name)
        .display()
        .to_string();
    #[cfg(windows)]
    let app_path = app_path.trim_start_matches(r#"\\?\"#).replace('\\', "/");
    pack_cmd.env("DO_PACK_EXE", app_path);

    if package == "deb" {
        pack_cmd.env("DO_PACK_DEB_DEPENDS", pack_deb::depends());
    }

    pack_cmd
        .status()
        .success_or_die("cannot package, failed cargo zng res");
}

/// do build-r [--bleed] [--dev]
///    Compile miscomp-issue release profile+features
///
///    ARGS
///       --bleed - Build with nightly compiler optimizations.
///       --dev   - Build with dev profile and release features.
fn build_r(args: Vec<String>) {
    let (_, options, args) = split_args(&args, &[], &["--bleed", "--dev"], true, true);
    let bleed = options.contains_key("--bleed");
    let dev = options.contains_key("--dev");

    let mut cmd = std::process::Command::new("cargo");
    if bleed {
        cmd.arg("+nightly");
    }
    cmd.args([
        "build",
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

/// do run-r [--dev]
///    Compile and run the "portable" pack
///
///    ARGS
///       --dev   - Build with dev profile and release features.
fn run_r(mut args: Vec<String>) {
    let app_args = if let Some(i) = args.iter().position(|a| a == "--") {
        args.split_off(i)
    } else {
        vec![]
    };

    println!("pack portable");
    args.push("portable".to_owned());
    pack(args);

    let path = format!(
        "target/pack/portable/miscomp-issue{}",
        std::env::consts::EXE_SUFFIX
    );
    println!("\nrunning {path}");
    let s = cmd(&path, app_args)
        .status()
        .unwrap_or_die("cannot run app");
    if !s.success() {
        std::process::exit(s.code().unwrap_or(1));
    }
}

/// do build-ndk [--platform API-LEVEL] [--target TRIPLE] [--release] [--dev]
///    Compile miscomp-issue-mobile for Android using cargo-ndk
///
///    Default --platform is the latest installed
///    Default --target is all android targets installed
///    Default profile is 'dev'
///
///    ARGS
///       --release - Build with release profile and features
///       --dev     - Build with dev profile and release features
fn build_ndk(args: Vec<String>) {
    let (_, options, unknown_args) = split_args(
        &args,
        &[],
        &["--release", "--dev", "--platform", "--target"],
        false,
        true,
    );

    // avoid relative path, see issue https://github.com/bbqsrc/cargo-ndk/issues/139
    let output_dir = std::env::current_dir()
        .unwrap()
        .join("target/build-ndk")
        .display()
        .to_string();

    let mut args = vec![
        "ndk",
        "--manifest-path",
        "crates/miscomp-issue-mobile/Cargo.toml",
        "--output-dir",
        &output_dir,
    ];
    if let Some(p) = options.get("--platform") {
        args.extend_from_slice(&["--platform", p[0]]);
    }

    let installed_targets;
    if let Some(t) = options.get("--target") {
        for t in t {
            args.extend_from_slice(&["--target", t]);
        }
    } else {
        installed_targets = cmd("rustup", &["target", "list", "--installed"])
            .output()
            .success_or_die("cannot get installed targets");

        let mut any = false;
        for line in installed_targets.lines() {
            if line.contains("-android") {
                any = true;
                args.extend_from_slice(&["--target", line]);
            }
        }

        if !any {
            die!("no android target installed, rustup target add aarch64-linux-android")
        }
    }

    args.extend_from_slice(&["build"]);
    if options.contains_key("--release") {
        args.extend_from_slice(&["--release", "--no-default-features", "--features=release"]);
    } else if options.contains_key("--dev") {
        args.extend_from_slice(&["--no-default-features", "--features=release"]);
    }
    args.extend_from_slice(&unknown_args);

    let mut cmd = cmd("cargo", &args);
    // args required to build linkme
    cmd.env(
        "RUSTFLAGS",
        format!(
            "{} -Clink-arg=-z -Clink-arg=nostart-stop-gc",
            std::env::var("RUSTFLAGS").unwrap_or_default()
        ),
    );
    // LTO "fat" and "thin" have caused miscompilation for "aarch64-linux-android"
    // see https://github.com/zng-ui/zng/issues/488 for details.
    // cmd.env("CARGO_PROFILE_RELEASE_LTO", "false");
    let s = cmd.status().unwrap_or_die("cannot run cargo-ndk");
    if !s.success() {
        std::process::exit(s.code().unwrap_or(1));
    }
}

/// do update
///    Update dependencies and localization from dependencies
fn update(args: Vec<String>) {
    cmd("cargo", &["update"])
        .args(&args)
        .status()
        .success_or_die("cargo update failed");

    if args.is_empty() {
        // update l10n resources from external dependencies
        l10n(vec!["--no-local".to_owned(), "--no-pkg".to_owned()]);
    }
}

/// do test-apk
///    Called by ci.yml after Android setup
fn test_apk(_: Vec<String>) {
    let log = cmd("adb", &["shell", r#""logcat""#])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_die("cannot read logcat");

    cmd(
        "adb",
        &[
            "shell",
            "am",
            "start",
            "-n",
            "rszng.zng_project.miscomp_issue_mobile/android.app.NativeActivity",
        ],
    )
    .status()
    .success_or_die("cannot run apk");

    // we expect two logs "!!: Some(_)" and "!!: None" (the error).
    let mut test_run = 0;
    println!("analyzing logs..");

    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(20));
        die!("timeout, no test print on log after 20s");
    });

    for line in BufReader::new(log.stdout.unwrap()).lines() {
        let line = line.unwrap_or_die("cannot read logcat line");
        if line.contains("miscomp") {
            println!("{line}");

            if let Some(i) = line.find("!!:") {
                let line = line[i..].trim_start();
                test_run += 1;
                if test_run == 2 {
                    if line.starts_with("None") {
                        die!("miscompilation detected");
                    } else {
                        assert!(line.starts_with("Some("));
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}

/// do help
///    Prints this help
fn help(_: Vec<String>) {
    self::help::print();
}
