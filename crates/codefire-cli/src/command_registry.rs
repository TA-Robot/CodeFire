use serde_json::{json, Value};

use crate::compatibility::{
    support_manifest_json, BRANCH_LIST_SCHEMA, CAPABILITIES_SCHEMA, COMMAND_RESULT_SCHEMA,
    SHOW_RESULT_SCHEMA,
};

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
    capability("close", true, false),
    capability("discard", true, false),
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
    "close",
    "discard",
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
        "schema": CAPABILITIES_SCHEMA,
        "command_result_schema": COMMAND_RESULT_SCHEMA,
        "support": support_manifest_json(),
        "commands": COMMAND_CAPABILITIES.iter().map(command_capability_json).collect::<Vec<_>>(),
    })
}

fn command_capability_json(capability: &CommandCapability) -> Value {
    let mut entry = json!({
        "name": capability.name,
        "supports_json": capability.supports_json,
        "supports_metrics": capability.supports_metrics,
        "help_key": capability.help_key,
        "targets": command_targets(capability.name),
        "output_schema": capability.supports_json.then_some(COMMAND_RESULT_SCHEMA),
        "pagination": command_pagination(capability.name),
    });
    if let Some(data_schema) = command_data_schema(capability.name) {
        entry
            .as_object_mut()
            .expect("command capability is an object")
            .insert("data_schema".to_string(), json!(data_schema));
    }
    entry
}

fn command_data_schema(name: &str) -> Option<&'static str> {
    match name {
        "branch" => Some(BRANCH_LIST_SCHEMA),
        "show" => Some(SHOW_RESULT_SCHEMA),
        _ => None,
    }
}

fn command_pagination(name: &str) -> Value {
    if matches!(name, "list" | "request-list") {
        json!({
            "mode": "snapshot_cursor",
            "cursor_schema": "codefire.page_cursor.v1",
            "page_schema": "codefire.page.v1",
            "legacy_unpaged_limit": null,
            "recommended_limit": crate::pagination::DEFAULT_PAGE_LIMIT,
            "max_limit": crate::pagination::MAX_PAGE_LIMIT,
        })
    } else {
        json!({"mode": "none"})
    }
}

fn command_targets(name: &str) -> &'static [&'static str] {
    match name {
        "clone" | "upload" | "request-merge" | "request-list" | "request-review"
        | "request-apply" => &["cf", "cf+http", "cf+https"],
        "list" | "show" | "diff" => &["local", "cf", "cf+http", "cf+https"],
        "doctor" => &["local", "cf", "cf+http", "cf+https"],
        _ => &["local"],
    }
}
