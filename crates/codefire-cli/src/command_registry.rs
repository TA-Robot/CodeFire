use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandCapability {
    pub(crate) name: &'static str,
    pub(crate) supports_json: bool,
    pub(crate) supports_metrics: bool,
    pub(crate) help_key: &'static str,
}

pub(crate) const COMMAND_CAPABILITIES: &[CommandCapability] = &[
    capability("init", false, false),
    capability("import", true, false),
    capability("open", true, false),
    capability("branch", true, true),
    capability("status", true, true),
    capability("scan", true, true),
    capability("fire", true, false),
    capability("extinguish", true, false),
    capability("verify", true, true),
    capability("commit", true, false),
    capability("clone", true, false),
    capability("merge", true, false),
    capability("upload", true, false),
    capability("list", true, false),
    capability("request-merge", true, false),
    capability("request-list", true, false),
    capability("request-review", true, false),
    capability("request-apply", true, false),
    capability("show", true, false),
    capability("diff", true, false),
    capability("review-pack", true, false),
    capability("patch", true, false),
    capability("doctor", true, false),
    capability("storage", true, false),
    capability("evidence", true, false),
    capability("explain", true, false),
    capability("context", true, false),
    capability("migrate", true, false),
    capability("serve", false, false),
    capability("completion", false, false),
    capability("capabilities", true, false),
    capability("version", false, false),
];

pub(crate) const COMMAND_NAMES: &[&str] = &[
    "init",
    "import",
    "open",
    "branch",
    "status",
    "scan",
    "fire",
    "extinguish",
    "verify",
    "commit",
    "clone",
    "merge",
    "upload",
    "list",
    "request-merge",
    "request-list",
    "request-review",
    "request-apply",
    "show",
    "diff",
    "review-pack",
    "patch",
    "doctor",
    "storage",
    "evidence",
    "explain",
    "context",
    "migrate",
    "serve",
    "completion",
    "capabilities",
    "version",
];

const fn capability(
    name: &'static str,
    supports_json: bool,
    supports_metrics: bool,
) -> CommandCapability {
    CommandCapability {
        name,
        supports_json,
        supports_metrics,
        help_key: name,
    }
}

pub(crate) fn command_capability(name: &str) -> Option<&'static CommandCapability> {
    COMMAND_CAPABILITIES
        .iter()
        .find(|capability| capability.name == name)
}

pub(crate) fn supports_metrics(name: &str) -> bool {
    if name == "branch list" {
        return true;
    }
    command_capability(name).is_some_and(|capability| capability.supports_metrics)
}

pub(crate) fn capabilities_data_json() -> Value {
    json!({
        "type": "codefire_command_capabilities",
        "version": 1,
        "commands": COMMAND_CAPABILITIES.iter().map(|capability| {
            json!({
                "name": capability.name,
                "supports_json": capability.supports_json,
                "supports_metrics": capability.supports_metrics,
                "help_key": capability.help_key,
            })
        }).collect::<Vec<_>>(),
    })
}
