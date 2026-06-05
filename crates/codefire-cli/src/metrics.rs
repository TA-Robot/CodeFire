use crate::Status;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct CommandMetrics {
    command: &'static str,
    total_ms: u128,
    counters: Vec<MetricCounter>,
}

#[derive(Debug, Clone)]
struct MetricCounter {
    name: &'static str,
    value: usize,
}

pub(crate) fn status_metrics(elapsed: Duration, status: &Status) -> CommandMetrics {
    CommandMetrics {
        command: "status",
        total_ms: elapsed.as_millis(),
        counters: vec![
            counter("open_fires", status.open_fires),
            counter("state_known", usize::from(!status.state.is_empty())),
        ],
    }
}

pub(crate) fn scan_metrics(elapsed: Duration, scan: &codefire_core::ScanResult) -> CommandMetrics {
    CommandMetrics {
        command: "scan",
        total_ms: elapsed.as_millis(),
        counters: vec![
            counter("atoms", scan.atom_index.atoms.len()),
            counter("trace_links", scan.trace_graph.links.len()),
            counter("changed_atoms", scan.changed_atoms.len()),
            counter("open_fires", scan.open_fires.len()),
        ],
    }
}

pub(crate) fn verification_metrics(
    elapsed: Duration,
    verification: &codefire_core::Verification,
) -> CommandMetrics {
    CommandMetrics {
        command: "verify",
        total_ms: elapsed.as_millis(),
        counters: vec![
            counter("open_required_fires", verification.open_required_fires),
            counter(
                "missing_required_links",
                verification.missing_required_links.len(),
            ),
            counter("stale_resolutions", verification.stale_resolutions.len()),
            counter("duplicate_atom_ids", verification.duplicate_atom_ids.len()),
            counter("failed_checks", verification.failed_checks.len()),
        ],
    }
}

pub(crate) fn attach_metrics(mut data: Value, metrics: Option<&CommandMetrics>) -> Value {
    if let Some(metrics) = metrics {
        if let Some(object) = data.as_object_mut() {
            object.insert("metrics".to_string(), metrics_json(metrics));
        }
    }
    data
}

pub(crate) fn print_metrics(metrics: &CommandMetrics) {
    println!("CodeFire metrics");
    println!("command: {}", metrics.command);
    println!("total: {} ms", metrics.total_ms);
    println!("phase timings:");
    println!("  total_ms: {}", metrics.total_ms);
    println!("counters:");
    for counter in &metrics.counters {
        println!("  {}: {}", counter.name, counter.value);
    }
    println!("cache:");
    println!("  enabled: false");
}

fn metrics_json(metrics: &CommandMetrics) -> Value {
    json!({
        "type": "codefire_metrics",
        "version": 1,
        "command": metrics.command,
        "phase_timings": {
            "total_ms": metrics.total_ms,
        },
        "counters": metrics.counters.iter().map(counter_json).collect::<Vec<_>>(),
        "cache": {
            "enabled": false,
        },
    })
}

fn counter_json(counter: &MetricCounter) -> Value {
    json!({
        "name": counter.name,
        "value": counter.value,
    })
}

fn counter(name: &'static str, value: usize) -> MetricCounter {
    MetricCounter { name, value }
}
