// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use move_prover::{cli::Options, run_move_prover};
use std::env;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let options = Options::create_from_args(&args)?;
    options.setup_logging();
    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    run_move_prover(&mut error_writer, options)
}
