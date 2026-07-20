use serde_json::{json, Value};
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn codefire() -> &'static str {
    env!("CARGO_BIN_EXE_codefire-rs")
}

fn initialized_repo(root: &Path) -> PathBuf {
    let repo = root.join("healthy");
    let output = Command::new(codefire())
        .arg("init")
        .arg(&repo)
        .output()
        .expect("run codefire init");
    assert!(
        output.status.success(),
        "init failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    repo
}

fn incomplete_repo(root: &Path) -> PathBuf {
    let repo = root.join("unhealthy");
    fs::create_dir_all(repo.join(".codefire")).expect("create incomplete repository");
    repo
}

fn run(args: &[&str], repo: &Path, json: bool) -> Output {
    let mut command = Command::new(codefire());
    command.args(args).arg(repo);
    if json {
        command.arg("--json");
    }
    command.output().expect("run CodeFire command")
}

fn single_json_document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout must contain exactly one JSON document: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_common_envelope(value: &Value, command: &str, repo: &Path, exit_code: i64) {
    assert_eq!(value["schema"], "codefire.command_result.v1");
    assert_eq!(value["command"], command);
    assert_eq!(value["exit_code"], exit_code);
    assert_eq!(value["repo"], repo.to_string_lossy().as_ref());
}

#[test]
fn doctor_json_is_one_document_for_healthy_and_unhealthy_reports() {
    let temp = tempdir().unwrap();
    let healthy = initialized_repo(temp.path());
    let unhealthy = incomplete_repo(temp.path());

    let healthy_output = run(&["doctor", "--full"], &healthy, true);
    assert!(healthy_output.status.success());
    assert!(healthy_output.stderr.is_empty());
    let healthy_json = single_json_document(&healthy_output);
    assert_common_envelope(&healthy_json, "doctor", &healthy, 0);
    assert_eq!(healthy_json["ok"], true);
    assert_eq!(healthy_json["data"]["type"], "codefire_doctor_report");
    assert_eq!(healthy_json["data"]["ok"], true);

    let unhealthy_output = run(&["doctor", "--full"], &unhealthy, true);
    assert_eq!(unhealthy_output.status.code(), Some(20));
    assert!(
        unhealthy_output.stderr.is_empty(),
        "structured failure must not be rendered again on stderr: {}",
        String::from_utf8_lossy(&unhealthy_output.stderr)
    );
    let unhealthy_json = single_json_document(&unhealthy_output);
    assert_common_envelope(&unhealthy_json, "doctor", &unhealthy, 20);
    assert_eq!(unhealthy_json["ok"], false);
    assert_eq!(unhealthy_json["data"]["type"], "codefire_doctor_report");
    assert_eq!(unhealthy_json["data"]["ok"], false);
    assert!(unhealthy_json["data"]["issues"].as_array().unwrap().len() > 1);
    assert!(!unhealthy_json["diagnostics"].as_array().unwrap().is_empty());
    assert!(!unhealthy_json["next_actions"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn remote_doctor_json_routes_file_url_and_returns_one_typed_document() {
    let temp = tempdir().unwrap();
    let target = format!("cf://{}/org/missing", temp.path().display());
    let output = Command::new(codefire())
        .args(["doctor", &target, "--quick", "--json"])
        .output()
        .expect("run remote doctor");

    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    let value = single_json_document(&output);
    assert_eq!(value["schema"], "codefire.command_result.v1");
    assert_eq!(value["command"], "doctor");
    assert_eq!(value["repo"], Value::Null);
    assert_eq!(value["data"]["schema"], "codefire.remote_doctor.v1");
    assert_eq!(value["data"]["transport"], "file");
    assert_eq!(value["data"]["project_exists"], false);
    assert_eq!(value["diagnostics"][0]["kind"], "remote_project_missing");
}

#[test]
fn rust_remote_doctor_consumes_python_http_report_contract() {
    let temp = tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("codefire");
    let mut server = Command::new("python3")
        .arg(script)
        .arg("serve")
        .arg(temp.path())
        .args(["--host", "127.0.0.1", "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        if Instant::now() >= deadline {
            let _ = server.kill();
            panic!("Python fallback HTTP server did not start");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let target = format!("cf+http://127.0.0.1:{port}/org/missing");
    let output = Command::new(codefire())
        .args(["doctor", &target, "--quick", "--json"])
        .output()
        .expect("run Rust doctor against Python HTTP server");
    let _ = server.kill();
    let _ = server.wait();

    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    let value = single_json_document(&output);
    assert_eq!(value["data"]["schema"], "codefire.remote_doctor.v1");
    assert_eq!(value["data"]["project"], target);
    assert_eq!(value["data"]["transport"], "http");
    assert_eq!(value["data"]["project_exists"], false);
    assert_eq!(value["data"]["quick"], true);
    assert_eq!(
        value["data"]["skipped_checks"],
        json!(["object_hash_and_reference_graph"])
    );
}

#[test]
fn migrate_json_is_one_document_for_compatible_and_incompatible_reports() {
    let temp = tempdir().unwrap();
    let healthy = initialized_repo(temp.path());
    let unhealthy = incomplete_repo(temp.path());

    for mode in ["check", "dry-run", "apply"] {
        let compatible_output = run(&["migrate", mode, "--full"], &healthy, true);
        assert!(
            compatible_output.status.success(),
            "compatible migrate {mode} failed: {}",
            String::from_utf8_lossy(&compatible_output.stderr)
        );
        assert!(compatible_output.stderr.is_empty());
        let compatible_json = single_json_document(&compatible_output);
        assert_common_envelope(&compatible_json, "migrate", &healthy, 0);
        assert_eq!(compatible_json["ok"], true);
        assert_eq!(compatible_json["data"]["type"], "codefire_migration_report");
        assert_eq!(compatible_json["data"]["mode"], mode);
        assert_eq!(compatible_json["data"]["compatible"], true);

        let incompatible_output = run(&["migrate", mode, "--full"], &unhealthy, true);
        assert_eq!(incompatible_output.status.code(), Some(40));
        assert!(
            incompatible_output.stderr.is_empty(),
            "structured failure must not be rendered again on stderr: {}",
            String::from_utf8_lossy(&incompatible_output.stderr)
        );
        let incompatible_json = single_json_document(&incompatible_output);
        assert_common_envelope(&incompatible_json, "migrate", &unhealthy, 40);
        assert_eq!(incompatible_json["ok"], false);
        assert_eq!(
            incompatible_json["data"]["type"],
            "codefire_migration_report"
        );
        assert_eq!(incompatible_json["data"]["mode"], mode);
        assert_eq!(incompatible_json["data"]["compatible"], false);
        assert!(!incompatible_json["data"]["blockers"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(!incompatible_json["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(!incompatible_json["next_actions"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}

#[test]
fn doctor_and_migrate_text_failures_keep_one_human_report_and_nonzero_exit() {
    let temp = tempdir().unwrap();
    let unhealthy = incomplete_repo(temp.path());

    let doctor = run(&["doctor", "--full"], &unhealthy, false);
    assert_eq!(doctor.status.code(), Some(20));
    let doctor_stdout = String::from_utf8(doctor.stdout).unwrap();
    let doctor_stderr = String::from_utf8(doctor.stderr).unwrap();
    assert_eq!(doctor_stdout.matches("CodeFire doctor").count(), 1);
    assert!(doctor_stdout.contains("ok: false"));
    assert!(doctor_stdout.contains("blocking issues:"));
    assert!(doctor_stderr.contains("doctor found repository problems"));

    for mode in ["check", "dry-run", "apply"] {
        let migrate = run(&["migrate", mode, "--full"], &unhealthy, false);
        assert_eq!(migrate.status.code(), Some(40));
        let migrate_stdout = String::from_utf8(migrate.stdout).unwrap();
        let migrate_stderr = String::from_utf8(migrate.stderr).unwrap();
        assert_eq!(migrate_stdout.matches("CodeFire migration").count(), 1);
        assert!(migrate_stdout.contains(&format!("CodeFire migration {mode}")));
        assert!(migrate_stdout.contains("compatible: false"));
        assert!(migrate_stdout.contains("blockers:"));
        assert!(migrate_stderr.contains("migration compatibility check failed"));
    }
}
