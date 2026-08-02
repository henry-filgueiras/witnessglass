//! WitnessGlass: a flight recorder for coding agents.
//!
//! This binary exists so the bootstrap repository compiles and is checkable. It
//! deliberately exposes no command surface, because no recorder exists yet. It
//! reports that honestly and exits with failure rather than implying capability
//! it does not have.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("witnessglass: bootstrap only; recording is not implemented.");
    eprintln!(
        "No session can be recorded, replayed, or inspected yet, and no command surface exists."
    );
    eprintln!("See README.md and the Scarp archaeology under archaeology/ for current scope.");
    ExitCode::FAILURE
}
