//! CLI entry for the privileged helper binary.
//!
//! Usage:
//!   hertzline-helper <op> [--reason TEXT] [--store PATH] [--json]
//!
//! Ops are allow-listed. Unknown tokens exit 2 without elevation.
//! `--store` sets `HERTZLINE_SNAPSHOT_STORE` for restore_all (uninstall/tests).

use hertzline_helper::{
    dispatch, elevate, parse_op_token,
    protocol::{HelperOp, HelperRequest, HelperResponse},
    restore, CRATE_VERSION,
};
use std::env;
use std::process::ExitCode;

fn print_usage() {
    eprintln!(
        "hertzline-helper {CRATE_VERSION}\n\
         usage: hertzline-helper <ping|probe_elevate_path|restore_all> [--reason TEXT] [--store PATH] [--json]\n\
         unknown ops are rejected (exit 2)"
    );
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return if args.is_empty() {
            ExitCode::from(2)
        } else {
            ExitCode::SUCCESS
        };
    }

    let json = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");

    let mut reason: Option<String> = None;
    let mut store: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--reason" {
            if i + 1 >= args.len() {
                eprintln!("error: --reason needs a value");
                return ExitCode::from(2);
            }
            reason = Some(args[i + 1].clone());
            args.drain(i..=i + 1);
            continue;
        }
        if args[i] == "--store" {
            if i + 1 >= args.len() {
                eprintln!("error: --store needs a value");
                return ExitCode::from(2);
            }
            store = Some(args[i + 1].clone());
            args.drain(i..=i + 1);
            continue;
        }
        i += 1;
    }

    if args.len() != 1 {
        print_usage();
        return ExitCode::from(2);
    }

    let op = match parse_op_token(&args[0]) {
        Ok(op) => op,
        Err(err) => {
            let detail = match err {
                hertzline_helper::protocol::HelperError::Rejected { detail } => detail,
            };
            let res = HelperResponse::rejected(detail);
            emit(&res, json);
            return ExitCode::from(2);
        }
    };

    if let Some(path) = store {
        env::set_var(restore::env_store_override(), path);
    }

    // Resolve launch plan for observability; elevation is not auto-prompted here.
    let _launch = elevate::resolve_launch(op);

    // restore_all: partial (missing handlers) stays accepted so uninstall can
    // continue after logging; hard load errors reject (exit 1).
    let res = if op == HelperOp::RestoreAll {
        let trigger = reason.as_deref().unwrap_or("explicit");
        let report = restore::restore_all(trigger);
        let mut response = HelperResponse::ok_json("restore_all", &report);
        if report.status == "error" {
            response.accepted = false;
        }
        response
    } else {
        dispatch(&HelperRequest { op, reason })
    };
    emit(&res, json);
    if res.accepted {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn emit(res: &HelperResponse, json: bool) {
    if json {
        match serde_json::to_string(res) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("serialize error: {e}"),
        }
        return;
    }
    if res.accepted {
        println!("ok {}", res.detail);
        if let Some(payload) = &res.payload {
            println!("{payload}");
        }
    } else {
        eprintln!("rejected {}", res.detail);
    }
}
