//! Per-command help text and help-flag detection.

pub(crate) fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}

pub(crate) fn command_name_for_args(args: &[String]) -> &str {
    match args.first().map(String::as_str) {
        Some("branch") => "branch",
        Some("patch") => "patch",
        Some("evidence") => "evidence",
        Some(command) => command,
        None => "help",
    }
}

pub(crate) fn command_help_for_args(args: &[String]) -> Option<&'static str> {
    let command = args.first()?.as_str();
    if !wants_help(&args[1..]) {
        return None;
    }
    match command {
        "status" | "scan" | "verify" | "fire" | "extinguish" | "commit" | "init" | "import"
        | "open" | "clone" | "close" | "discard" | "merge" | "upload" | "list"
        | "request-merge" | "request-list" | "request-review" | "request-apply" | "review-pack"
        | "storage" | "doctor" | "show" | "diff" | "context" | "explain" | "atom-index"
        | "trace-graph" | "missing-links" | "link" | "capabilities" => {
            Some(subcommand_help(command))
        }
        "migrate" => match args.get(1).map(String::as_str) {
            Some("check") | Some("dry-run") | Some("apply") => {
                Some(subcommand_help("migrate check"))
            }
            _ => Some(subcommand_help("migrate")),
        },
        "branch" => match args.get(1).map(String::as_str) {
            Some("list") => Some(subcommand_help("branch list")),
            Some("show") => Some(subcommand_help("branch show")),
            _ => Some(subcommand_help("branch")),
        },
        "evidence" => match args.get(1).map(String::as_str) {
            Some("add") => Some(subcommand_help("evidence add")),
            _ => Some(subcommand_help("evidence")),
        },
        "patch" => match args.get(1).map(String::as_str) {
            Some("export") => Some(subcommand_help("patch export")),
            Some("import") => Some(subcommand_help("patch import")),
            _ => Some(subcommand_help("patch")),
        },
        _ => None,
    }
}

pub(crate) fn subcommand_help(command: &str) -> &'static str {
    match command {
        "init" => {
            "usage: codefire init [path] [--force]\n\nCreate a CodeFire repository without opening a branch.\n"
        }
        "import" => {
            "usage: codefire import [path] [--branch <name>] [--dry-run] [--json] [--idempotency-key <key>]\n\nAdopt an existing non-empty directory as a CodeFire open directory.\n"
        }
        "open" => {
            "usage: codefire open <branch> <path> [--dry-run] [--json] [--idempotency-key <key>]\n\nOpen a sealed branch into a working directory.\n\n<path> is the directory to work in. It must be an absolute path outside the repository root, beside it: for a repository at /work/repo, use for example /work/<branch>. A branch is open in at most one directory; `codefire close <branch>` removes it again.\n"
        }
        "clone" => {
            "usage: codefire clone <source-branch> <new-branch> [--dry-run] [--json] [--idempotency-key <key>]\n\nCreate a new sealed branch from an existing branch. This is how a branch is created.\n\nThe new branch starts closed. Run `codefire open <new-branch> <path>` to get a working directory for it.\n"
        }
        "close" => {
            "usage: codefire close <branch> [--discard] [--keep-dir] [--dry-run] [--json]\n\nClose an open-clean branch and remove its open directory. Use --discard to explicitly discard unsealed open work.\n\nClosing loses nothing: the sealed commits stay and the branch can be opened again. close refuses while the directory holds unsealed work; commit it first, or throw it away with `codefire discard <branch>`.\n"
        }
        "discard" => {
            "usage: codefire discard <branch> [--keep-dir] [--dry-run] [--json]\n\nDiscard an open branch worktree and close its open registry entry.\n\nThis throws away every change since the last sealed commit of the branch. For a branch with nothing unsealed, use `codefire close <branch>` instead.\n"
        }
        "merge" => {
            "usage: codefire merge <source-branch> --into <target-branch> [--dry-run] [--json] [--idempotency-key <key>]\n       codefire merge --continue [path|--path <open-dir>] [--accept-semantic-conflicts] [--dry-run] [--json]\n\nMerge a sealed source branch head into an open-clean target branch, then explicitly continue after resolving conflicts.\n\nText files changed on both branches are merged line by line. A region both branches changed differently is written between `<<<<<<< target`, `=======` and `>>>>>>> source` marker lines: edit the file, keep the lines you want, delete the marker lines.\n`merge --continue` (run in the target open directory) lists what is still unresolved and records the resolution when nothing is. An atom that both branches changed is a semantic conflict: review the merged result, then pass --accept-semantic-conflicts to record that you reviewed it. It does not skip verification; scan, extinguish, verify and commit still follow.\n"
        }
        "upload" => {
            "usage: codefire upload <branch> <remote-url> [--dry-run] [--json] [--idempotency-key <key>] [--request-key-id <key>]\n\nUpload a sealed branch to a remote project.\n"
        }
        "list" => {
            "usage: codefire list [path|<remote-project-url>] [--json] [--limit N] [--cursor CURSOR]\n\nList local repository branches/open worktrees or remote project branches. Pagination options require a remote target and bind the cursor to one immutable result snapshot.\n"
        }
        "request-merge" => {
            "usage: codefire request-merge <source-url> <target-url> [--dry-run] [--json] [--idempotency-key <key>]\n\nCreate a remote merge request.\n"
        }
        "request-list" => {
            "usage: codefire request-list <remote-project-url> [--json] [--limit N] [--cursor CURSOR]\n\nList remote merge requests with a snapshot-bound cursor.\n"
        }
        "request-review" => {
            "usage: codefire request-review <remote-project-url> <mr-id> --decision approve|reject [--reviewer <name>] [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "request-apply" => {
            "usage: codefire request-apply <remote-project-url> <mr-id> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "status" => {
            "usage: codefire status [path|--path <path>] [--json] [--metrics]\n\nShow the current open branch state.\n"
        }
        "scan" => {
            "usage: codefire scan [path|--path <path>] [--json] [--metrics] [--full] [--wait-lock] [--lock-timeout <duration>]\n\nDetect changed atoms and open fires for an open directory.\n"
        }
        "verify" => {
            "usage: codefire verify [path|--path <path>] [--details] [--blocking-only] [--json] [--metrics] [--wait-lock] [--lock-timeout <duration>]\n\nRun CodeFire verification checks for an open directory.\n"
        }
        "fire" => {
            "usage: codefire fire <source-atom> --to <target-atom> --reason <text> [--path <open-dir>] [--dry-run] [--full] [--json]\n       codefire fire --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"fires\":[{\"from\":\"REQ-id\",\"to\":\"DES-id\",\"reason\":\"manual review\"}]}\n"
        }
        "extinguish" => {
            "usage: codefire extinguish <fire-id> [--path <open-dir>] [--refresh] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>) [--dry-run] [--json]\n       codefire extinguish --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]\n       codefire extinguish --batch-template [--path <open-dir>] [--resolution <type>] [--rationale <text>|--evidence <text>|--evidence-ref <id>] [--output <file>] [--json]\n\n--resolution is an extensible string, not a closed enum; omitting it defaults to addressed. Common examples: addressed, changed, no-change-required. Supply at least one of --rationale, --evidence, or --evidence-ref.\n--refresh re-records the resolution of an already extinguished fire against the current atoms, trace links, and policy; use it for each fire that verify reports as a stale resolution.\nJSON next_actions may include optional execution metadata (schema codefire.action_execution.v1): ready=true actions expose a complete argv for exec without a shell; ready=false actions require named required_inputs before they are executable. The command string is a compatibility projection of that argv or argv_template.\n\nBatch JSON example:\n  {\"version\":1,\"fires\":[{\"id\":\"FIRE-1\",\"resolution\":\"addressed\",\"rationale\":\"fixed\"}]}\n"
        }
        "commit" => {
            "usage: codefire commit [path|--path <open-dir>] -m <message> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "branch" => {
            "usage: codefire branch list [path|--path <repo-or-open>] [--json] [--metrics]\n       codefire branch show [branch] [path|--path <repo-or-open>] [--json]\n\nInspect local branches.\n\nCreate a branch with `codefire clone <source-branch> <new-branch>`, work on it with `codefire open <new-branch> <path>`, bring it into another branch with `codefire merge <source-branch> --into <target-branch>`, and stop working on it with `codefire close <branch>`.\n"
        }
        "branch list" => {
            "usage: codefire branch list [path|--path <repo-or-open>] [--json] [--metrics]\n\nList local branches.\n"
        }
        "branch show" => {
            "usage: codefire branch show [branch] [path|--path <repo-or-open>] [--json]\n\nShow a local branch detail record.\n"
        }
        "context" => {
            "usage: codefire context (--changed|--atom <atom-id>|--fire <fire-id>|--branch <branch>) [--path <open-dir>] [--depth <n>] [--limit <n>] [--json]\n\nBuild a bounded context pack for an open directory.\n"
        }
        "explain" => {
            "usage: codefire explain (fire <fire-id>|atom <atom-id>|verify-failure|storage-warning) [--path <open-dir>] [--json]\n\nExplain CodeFire state, diagnostics, and next actions.\n"
        }
        "migrate" | "migrate check" => {
            "usage: codefire migrate check [path] [--json] [--target-format current|v0.6] [--quick|--full]\n       codefire migrate dry-run [path] [--json] [--target-format current|v0.6] [--quick|--full]\n       codefire migrate apply [path] [--json] [--target-format current|v0.6] [--full] [--wait-lock] [--lock-timeout <duration>]\n\nCheck, plan, or apply registered layout-only repository repairs. Target formats select a compatibility profile; migrate does not rewrite repository format metadata.\n"
        }
        "atom-index" => {
            "usage: codefire atom-index [path]\n\nPrint the current Atom index as JSON for a repository or open directory.\n"
        }
        "trace-graph" => {
            "usage: codefire trace-graph [path]\n\nPrint the current trace graph as JSON for a repository or open directory.\n"
        }
        "missing-links" => {
            "usage: codefire missing-links [path]\n\nPrint required trace links that are currently missing.\n"
        }
        "link" => {
            "usage: codefire link --batch <file> [--path <open-dir>] [--dry-run] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"links\":[{\"from\":\"REQ-id\",\"to\":\"DES-id\",\"type\":\"refined_by\"}]}\n"
        }
        "storage" => {
            "usage: codefire storage [path] [--quick|--full] [--json] [--large-threshold <bytes|KB|MB|GB>] [--remote <cf://server/org/app>]\n       codefire storage report [path] [--quick|--full] [--json] [--large-threshold <bytes|KB|MB|GB>] [--remote <cf://server/org/app>]\n"
        }
        "doctor" => {
            "usage: codefire doctor [path] [--quick|--full] [--json]\n\nInspect repository health.\n"
        }
        "evidence" | "evidence add" => {
            "usage: codefire evidence add [--path <repo-or-open>] (--artifact <path>|--from-command <command>|--from-argv <program> [--argv <arg>...]|--batch <file>) [--label <text>] [--cwd <dir>] [--timeout <duration>] [--max-output-bytes <bytes>] [--allow-failed-command] [--dry-run] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"items\":[{\"from_command\":\"cargo test --workspace\",\"label\":\"tests\"}]}\n"
        }
        "show" => {
            "usage: codefire show <branch-or-commit> [--json]\n\nShow a sealed commitish.\n"
        }
        "diff" => {
            "usage: codefire diff [--algorithm myers|patience|histogram] [--context <lines>] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <left> <right>\n"
        }
        "review-pack" => {
            "usage: codefire review-pack <source> [--base <base>] [--output <path>] [--algorithm myers|patience|histogram] [--no-rename-detection] [--json]\n"
        }
        "patch" => {
            "usage: codefire patch export <source> [--base <base>] [--output <path>]\n       codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "patch export" => {
            "usage: codefire patch export <source> [--base <base>] [--output <path>] [--max-file-bytes <bytes>] [--max-payload-bytes <bytes>] [--include-large-files] [--json]\n"
        }
        "patch import" => {
            "usage: codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "capabilities" => {
            "usage: codefire capabilities [--json]\n\nList CLI command automation capabilities.\n"
        }
        _ => "usage: codefire <command> [options]\n",
    }
}
