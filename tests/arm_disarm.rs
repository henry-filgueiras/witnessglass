//! `scripts/arm.sh` and `scripts/disarm.sh`.
//!
//! Every test here runs against a throwaway directory that merely *looks* like
//! the repository — scripts, the example configuration, and a copy of the built
//! binary. The real repository is never armed by the test suite, because arming
//! it would install hooks on the machine running the tests.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Build a throwaway tree with the same shape the scripts expect.
fn fake_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));

    std::fs::create_dir_all(root.join("scripts")).expect("scripts");
    std::fs::create_dir_all(root.join(".claude")).expect(".claude");
    std::fs::create_dir_all(root.join("target/debug")).expect("target");

    for script in ["arm.sh", "disarm.sh"] {
        std::fs::copy(
            repo.join("scripts").join(script),
            root.join("scripts").join(script),
        )
        .expect("copy script");
    }
    std::fs::copy(
        repo.join(".claude/settings.witnessglass.example.json"),
        root.join(".claude/settings.witnessglass.example.json"),
    )
    .expect("copy example");
    std::fs::copy(
        env!("CARGO_BIN_EXE_witnessglass"),
        root.join("target/debug/witnessglass"),
    )
    .expect("copy binary");

    dir
}

fn run(root: &Path, script: &str, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(root.join("scripts").join(script))
        .args(args)
        .output()
        .expect("run script")
}

/// Arming without `--no-build` would need a Cargo project in the fake tree.
fn arm(root: &Path) -> Output {
    run(root, "arm.sh", &["--no-build"])
}

fn disarm(root: &Path) -> Output {
    run(root, "disarm.sh", &[])
}

fn settings(root: &Path) -> PathBuf {
    root.join(".claude/settings.local.json")
}

fn sentinel(root: &Path) -> PathBuf {
    root.join(".witnessglass/armed")
}

fn assert_ok(output: &Output, what: &str) {
    assert!(
        output.status.success(),
        "{what} failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Files matching a prefix inside `.claude`, so the aside/displaced copies can
/// be found without guessing their timestamps.
fn claude_files_starting_with(root: &Path, prefix: &str) -> Vec<String> {
    std::fs::read_dir(root.join(".claude"))
        .expect("read .claude")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.starts_with(prefix))
        .collect()
}

#[test]
fn arming_installs_the_configuration_and_a_sentinel() {
    let dir = fake_repo();
    let root = dir.path();

    assert!(!settings(root).exists());
    assert_ok(&arm(root), "arm");

    // The configuration Claude actually reads is in place and is the example.
    let installed = std::fs::read_to_string(settings(root)).expect("settings");
    let example = std::fs::read_to_string(root.join(".claude/settings.witnessglass.example.json"))
        .expect("example");
    assert_eq!(installed, example);

    // The sentinel records what arming did, not merely that it happened.
    let recorded = std::fs::read_to_string(sentinel(root)).expect("sentinel");
    for key in [
        "armed_at=",
        "binary=",
        "binary_sha256=",
        "settings_sha256=",
        "recordings_dir=",
    ] {
        assert!(recorded.contains(key), "sentinel missing {key}: {recorded}");
    }
    // Nothing was displaced, and the sentinel says so rather than staying silent.
    assert!(recorded.contains("displaced_settings=\n"), "{recorded}");

    // The self-test recording went to a throwaway directory, not into the tree.
    let recordings = root.join(".witnessglass/recordings");
    assert!(recordings.is_dir(), "recordings directory should exist");
    assert_eq!(std::fs::read_dir(&recordings).expect("read").count(), 0);
}

#[test]
fn disarming_removes_everything_arming_added() {
    let dir = fake_repo();
    let root = dir.path();

    assert_ok(&arm(root), "arm");
    assert_ok(&disarm(root), "disarm");

    assert!(!settings(root).exists(), "settings should be gone");
    assert!(!sentinel(root).exists(), "sentinel should be gone");
    // The example is never touched.
    assert!(
        root.join(".claude/settings.witnessglass.example.json")
            .exists()
    );
}

#[test]
fn disarming_when_not_armed_is_a_no_op() {
    let dir = fake_repo();
    let root = dir.path();

    let output = disarm(root);
    assert_ok(&output, "disarm");
    assert!(String::from_utf8_lossy(&output.stdout).contains("not armed"));

    // And it stays idempotent after a real arm/disarm cycle.
    assert_ok(&arm(root), "arm");
    assert_ok(&disarm(root), "disarm");
    assert_ok(&disarm(root), "second disarm");
}

#[test]
fn re_arming_disarms_first_rather_than_stacking() {
    let dir = fake_repo();
    let root = dir.path();

    assert_ok(&arm(root), "first arm");
    let first = std::fs::read_to_string(sentinel(root)).expect("sentinel");

    let second_output = arm(root);
    assert_ok(&second_output, "second arm");
    assert!(
        String::from_utf8_lossy(&second_output.stdout).contains("already armed"),
        "re-arm should say it is re-arming"
    );

    // Exactly one configuration and one sentinel, both freshly written.
    assert!(settings(root).exists());
    let second = std::fs::read_to_string(sentinel(root)).expect("sentinel");
    assert!(second.contains("armed_at="));

    // No displaced copy was created: re-arming removed its own configuration
    // rather than treating it as a stranger's and moving it aside.
    assert!(
        claude_files_starting_with(root, "settings.local.json.").is_empty(),
        "re-arming left debris: {:?}",
        claude_files_starting_with(root, "settings.local.json.")
    );
    assert!(first.contains("armed_at=") && second.contains("armed_at="));
}

#[test]
fn re_arming_recovers_when_the_sentinel_was_deleted() {
    // A deleted sentinel must not strand an armed configuration that nothing
    // knows how to remove.
    let dir = fake_repo();
    let root = dir.path();

    assert_ok(&arm(root), "arm");
    std::fs::remove_file(sentinel(root)).expect("delete sentinel");

    let output = arm(root);
    assert_ok(&output, "re-arm");
    assert!(String::from_utf8_lossy(&output.stdout).contains("already armed"));
    assert!(sentinel(root).exists());
    assert!(
        claude_files_starting_with(root, "settings.local.json.").is_empty(),
        "recovery should not have displaced our own configuration"
    );
}

#[test]
fn a_pre_existing_settings_file_is_displaced_and_then_restored() {
    // The hazard the sentinel exists for. Someone's own settings.local.json
    // must survive an arm/disarm cycle intact.
    let dir = fake_repo();
    let root = dir.path();

    let mine = r#"{"env":{"MY_OWN_SETTING":"synthetic"}}"#;
    std::fs::write(settings(root), mine).expect("write mine");

    let armed = arm(root);
    assert_ok(&armed, "arm");
    assert!(String::from_utf8_lossy(&armed.stdout).contains("moved your existing"));

    // Armed with ours; theirs is safely aside and named for what happened.
    assert!(
        std::fs::read_to_string(settings(root))
            .unwrap()
            .contains("claude-hook")
    );
    let displaced =
        claude_files_starting_with(root, "settings.local.json.displaced-by-witnessglass.");
    assert_eq!(displaced.len(), 1, "expected exactly one displaced copy");

    assert_ok(&disarm(root), "disarm");

    // Returned byte-for-byte, and the aside copy is consumed rather than left
    // lying around.
    assert_eq!(std::fs::read_to_string(settings(root)).unwrap(), mine);
    assert!(
        claude_files_starting_with(root, "settings.local.json.displaced-by-witnessglass.")
            .is_empty(),
        "the displaced copy should have been moved back, not copied"
    );
    assert!(!sentinel(root).exists());
}

#[test]
fn an_edited_configuration_is_moved_aside_rather_than_deleted() {
    // Disarm never deletes a file it did not write byte-for-byte.
    let dir = fake_repo();
    let root = dir.path();

    assert_ok(&arm(root), "arm");
    let mut edited = std::fs::read_to_string(settings(root)).expect("settings");
    edited.push('\n');
    std::fs::write(settings(root), &edited).expect("edit");

    let output = disarm(root);
    assert_ok(&output, "disarm");
    assert!(String::from_utf8_lossy(&output.stdout).contains("changed since arming"));

    assert!(!settings(root).exists(), "should be disarmed");
    let aside = claude_files_starting_with(root, "settings.local.json.disarmed.");
    assert_eq!(aside.len(), 1, "the edit should have been preserved");
    assert_eq!(
        std::fs::read_to_string(root.join(".claude").join(&aside[0])).unwrap(),
        edited
    );
}

#[test]
fn a_foreign_settings_file_is_left_untouched_by_disarm() {
    let dir = fake_repo();
    let root = dir.path();

    let theirs = r#"{"env":{"NOT_OURS":"synthetic"}}"#;
    std::fs::write(settings(root), theirs).expect("write");

    let output = disarm(root);
    assert_ok(&output, "disarm");
    assert!(String::from_utf8_lossy(&output.stdout).contains("not a WitnessGlass configuration"));
    assert_eq!(std::fs::read_to_string(settings(root)).unwrap(), theirs);
}

#[test]
fn disarming_keeps_recordings() {
    // Disarming stops recording. It does not discard evidence already captured.
    let dir = fake_repo();
    let root = dir.path();

    assert_ok(&arm(root), "arm");

    let recording = root.join(".witnessglass/recordings/synthetic-session.ndjson");
    std::fs::write(&recording, "{\"schema_version\":2}\n").expect("write recording");

    let output = disarm(root);
    assert_ok(&output, "disarm");
    assert!(recording.exists(), "recordings must survive a disarm");
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 recording(s) kept"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("NOT safe to share"));
}

#[test]
fn arming_refuses_when_the_binary_fails_its_self_test() {
    // A broken adapter must be found before a session, not halfway through the
    // one it was supposed to record.
    let dir = fake_repo();
    let root = dir.path();

    std::fs::write(
        root.join("target/debug/witnessglass"),
        "#!/bin/sh\nexit 3\n",
    )
    .expect("write stub");

    let output = arm(root);
    assert!(!output.status.success(), "arming should have refused");
    assert!(String::from_utf8_lossy(&output.stderr).contains("self-test"));
    assert!(
        !settings(root).exists(),
        "a failed arm must not install hooks"
    );
    assert!(!sentinel(root).exists());
}

#[test]
fn arming_refuses_when_the_binary_writes_to_stdout() {
    // Anything on stdout is read by Claude as a decision, which would make the
    // adapter capable of influencing the session it records.
    let dir = fake_repo();
    let root = dir.path();

    std::fs::write(
        root.join("target/debug/witnessglass"),
        "#!/bin/sh\necho '{\"decision\":\"block\"}'\nexit 0\n",
    )
    .expect("write stub");

    let output = arm(root);
    assert!(!output.status.success(), "arming should have refused");
    assert!(String::from_utf8_lossy(&output.stderr).contains("stdout"));
    assert!(!settings(root).exists());
}
