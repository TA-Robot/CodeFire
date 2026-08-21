use crate::CliError;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const READ_CHUNK_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct CommandExecutionSpec {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) clear_env: bool,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) timeout_ms: u64,
    pub(crate) stdout_limit: usize,
    pub(crate) stderr_limit: usize,
}

#[derive(Debug)]
pub(crate) struct BoundedStream {
    pub(crate) text: String,
    pub(crate) bytes_seen: u64,
    pub(crate) bytes_stored: usize,
    pub(crate) truncated: bool,
}

#[derive(Debug)]
pub(crate) struct CommandExecution {
    pub(crate) exit_code: Option<i32>,
    pub(crate) success: bool,
    pub(crate) timed_out: bool,
    pub(crate) duration_ms: u128,
    pub(crate) stdout: BoundedStream,
    pub(crate) stderr: BoundedStream,
    pub(crate) termination_scope: &'static str,
}

pub(crate) fn minimal_command_environment() -> BTreeMap<String, String> {
    let mut environment = BTreeMap::from([("PATH".to_string(), safe_default_path())]);
    environment.extend(["LANG", "LC_ALL", "TZ"].into_iter().filter_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| (key.to_string(), value))
    }));
    environment
}

pub(crate) fn canonical_digest(value: &Value) -> Result<String, CliError> {
    Ok(format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(value)?)
    ))
}

#[cfg(unix)]
fn safe_default_path() -> String {
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()
}

#[cfg(not(unix))]
fn safe_default_path() -> String {
    std::env::var("PATH").unwrap_or_default()
}

pub(crate) fn run_bounded_command(
    spec: &CommandExecutionSpec,
) -> Result<CommandExecution, CliError> {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .current_dir(&spec.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if spec.clear_env {
        command.env_clear();
    }
    command.envs(&spec.env);
    configure_process_group(&mut command);

    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CliError::Io(io::Error::other("bounded command stdout pipe is missing")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CliError::Io(io::Error::other("bounded command stderr pipe is missing")))?;
    configure_pipe_nonblocking(&stdout)?;
    configure_pipe_nonblocking(&stderr)?;
    let stop_readers = Arc::new(AtomicBool::new(false));
    let stdout_reader = spawn_bounded_reader(stdout, spec.stdout_limit, Arc::clone(&stop_readers));
    let stderr_reader = spawn_bounded_reader(stderr, spec.stderr_limit, Arc::clone(&stop_readers));

    let started = Instant::now();
    let timeout = Duration::from_millis(spec.timeout_ms);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            terminate_process_tree(&mut child);
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let duration_ms = started.elapsed().as_millis();
    stop_readers.store(true, Ordering::Release);
    let stdout = join_reader(stdout_reader)?;
    let stderr = join_reader(stderr_reader)?;

    Ok(CommandExecution {
        exit_code: status.code(),
        success: status.success() && !timed_out,
        timed_out,
        duration_ms,
        stdout,
        stderr,
        termination_scope: termination_scope(),
    })
}

fn spawn_bounded_reader<R>(
    mut reader: R,
    limit: usize,
    stop: Arc<AtomicBool>,
) -> JoinHandle<io::Result<BoundedStream>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut stored = Vec::with_capacity(limit.min(READ_CHUNK_BYTES));
        let mut bytes_seen = 0u64;
        let mut chunk = [0u8; READ_CHUNK_BYTES];
        loop {
            let count = match reader.read(&mut chunk) {
                Ok(count) => count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(2));
                    continue;
                }
                Err(error) => return Err(error),
            };
            if count == 0 {
                break;
            }
            bytes_seen = bytes_seen.saturating_add(count as u64);
            let remaining = limit.saturating_sub(stored.len());
            stored.extend_from_slice(&chunk[..count.min(remaining)]);
        }
        let bytes_stored = stored.len();
        Ok(BoundedStream {
            text: String::from_utf8_lossy(&stored).into_owned(),
            bytes_seen,
            bytes_stored,
            truncated: bytes_seen > bytes_stored as u64,
        })
    })
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
))]
fn configure_pipe_nonblocking<Fd: std::os::fd::AsFd>(fd: &Fd) -> io::Result<()> {
    use rustix::fs::{fcntl_getfl, fcntl_setfl, OFlags};

    let flags = fcntl_getfl(fd).map_err(io::Error::from)?;
    fcntl_setfl(fd, flags | OFlags::NONBLOCK).map_err(io::Error::from)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
)))]
fn configure_pipe_nonblocking<Fd>(_fd: &Fd) -> io::Result<()> {
    Ok(())
}

fn join_reader(reader: JoinHandle<io::Result<BoundedStream>>) -> Result<BoundedStream, CliError> {
    reader
        .join()
        .map_err(|_| CliError::Io(io::Error::other("bounded command reader panicked")))?
        .map_err(CliError::from)
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) {
    const SIGKILL: i32 = 9;
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    let process_group = -(child.id() as i32);
    // SAFETY: the child was placed in a new process group whose ID equals its PID. SIGKILL has no
    // borrowed-memory contract; failures (including an already-exited group) are handled by the
    // direct-child fallback below.
    unsafe {
        kill(process_group, SIGKILL);
    }
    let _ = child.kill();
}

#[cfg(not(unix))]
fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
}

#[cfg(unix)]
fn termination_scope() -> &'static str {
    "process_group"
}

#[cfg(not(unix))]
fn termination_scope() -> &'static str {
    "direct_child"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_spec(script: &str, limit: usize, timeout_ms: u64) -> CommandExecutionSpec {
        CommandExecutionSpec {
            program: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            cwd: std::env::temp_dir(),
            clear_env: true,
            env: BTreeMap::new(),
            timeout_ms,
            stdout_limit: limit,
            stderr_limit: limit,
        }
    }

    #[test]
    fn drains_output_while_storing_only_the_declared_cap() {
        let output = run_bounded_command(&shell_spec(
            "i=0; while [ $i -lt 20000 ]; do printf x; i=$((i+1)); done",
            37,
            5_000,
        ))
        .unwrap();
        assert!(output.success);
        assert_eq!(output.stdout.bytes_seen, 20_000);
        assert_eq!(output.stdout.bytes_stored, 37);
        assert_eq!(output.stdout.text.len(), 37);
        assert!(output.stdout.truncated);
    }

    #[test]
    fn timeout_terminates_and_drains_the_command() {
        let output =
            run_bounded_command(&shell_spec("while :; do printf x; done", 19, 20)).unwrap();
        assert!(!output.success);
        assert!(output.timed_out);
        assert_eq!(output.stdout.bytes_stored, 19);
        assert!(output.stdout.truncated);
    }

    #[cfg(unix)]
    #[test]
    fn timeout_terminates_descendants_and_drains_both_streams() {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("descendant-survived");
        let mut spec = shell_spec(
            "(sleep 0.15; printf survived > \"$MARKER\") & while :; do printf o; printf e >&2; done",
            23,
            30,
        );
        spec.env
            .insert("MARKER".to_string(), marker.to_string_lossy().into_owned());

        let output = run_bounded_command(&spec).unwrap();
        thread::sleep(Duration::from_millis(250));

        assert!(output.timed_out);
        assert_eq!(output.termination_scope, "process_group");
        assert_eq!(output.stdout.bytes_stored, 23);
        assert_eq!(output.stderr.bytes_stored, 23);
        assert!(output.stdout.truncated);
        assert!(output.stderr.truncated);
        assert!(!marker.exists());
    }

    #[test]
    fn minimal_environment_excludes_caller_secrets_and_uses_a_bounded_key_set() {
        let environment = minimal_command_environment();
        assert!(environment.contains_key("PATH"));
        assert!(environment
            .keys()
            .all(|key| matches!(key.as_str(), "PATH" | "LANG" | "LC_ALL" | "TZ")));
        assert!(!environment.contains_key("HOME"));
        assert!(!environment.contains_key("OPENAI_API_KEY"));
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[test]
    fn timeout_reader_does_not_wait_for_a_detached_pipe_holder() {
        let started = Instant::now();
        let output = run_bounded_command(&shell_spec(
            "setsid sh -c 'sleep 1' & while :; do printf x; done",
            17,
            20,
        ))
        .unwrap();

        assert!(output.timed_out);
        assert!(started.elapsed() < Duration::from_millis(500));
        assert_eq!(output.stdout.bytes_stored, 17);
    }
}
