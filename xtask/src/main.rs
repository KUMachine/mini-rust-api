//! Project automation, run as `cargo xtask <command>`

use std::path::PathBuf;
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <command>

Commands:
  check   Run every CI gate: format, clippy, tests, docs
  fix     Apply rustfmt and machine-applicable clippy suggestions
";

struct Step {
    name: &'static str,
    args: &'static [&'static str],
    env: &'static [(&'static str, &'static str)],
}

/// Exactly what CI runs; a change is done when this passes
const CHECK: &[Step] = &[
    Step {
        name: "format",
        args: &["fmt", "--all", "--", "--check"],
        env: &[],
    },
    Step {
        name: "clippy",
        args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        env: &[],
    },
    Step {
        name: "test",
        args: &["test", "--workspace", "--locked"],
        env: &[],
    },
    Step {
        name: "docs",
        args: &["doc", "--workspace", "--no-deps", "--all-features"],
        env: &[("RUSTDOCFLAGS", "-D warnings")],
    },
];

const FIX: &[Step] = &[
    Step {
        name: "clippy --fix",
        args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--fix",
            "--allow-dirty",
            "--allow-staged",
        ],
        env: &[],
    },
    Step {
        name: "format",
        args: &["fmt", "--all"],
        env: &[],
    },
];

fn main() -> ExitCode {
    let steps = match std::env::args().nth(1).as_deref() {
        Some("check") => CHECK,
        Some("fix") => FIX,
        _ => {
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    for step in steps {
        if let Err(message) = run(step) {
            eprintln!("\n✗ {} failed: {message}", step.name);
            eprintln!("  reproduce with: cargo {}", step.args.join(" "));
            return ExitCode::FAILURE;
        }
    }

    eprintln!("\n✓ all {} steps passed", steps.len());
    ExitCode::SUCCESS
}

fn run(step: &Step) -> Result<(), String> {
    eprintln!("\n==> {}: cargo {}", step.name, step.args.join(" "));

    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .args(step.args)
        .envs(step.env.iter().copied())
        .current_dir(workspace_root())
        .status()
        .map_err(|e| format!("could not start cargo: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("exited with {status}"))
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}
