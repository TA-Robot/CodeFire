use crate::Status;
use serde_json::{json, Map, Value};
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct CommandMetrics {
    command: &'static str,
    total_ms: u128,
    phases: Vec<MetricPhase>,
    counters: Vec<MetricCounter>,
    cache: CacheMetrics,
}

#[derive(Debug, Clone)]
struct MetricPhase {
    name: &'static str,
    elapsed_ms: u128,
    measured: bool,
}

#[derive(Debug, Clone)]
struct MetricCounter {
    name: &'static str,
    value: usize,
}

#[derive(Debug, Clone)]
struct CacheMetrics {
    status: &'static str,
    enabled: bool,
    implementation: &'static str,
    disabled_reason: &'static str,
    entry_count: usize,
    hit_count: usize,
    miss_count: usize,
}

pub(crate) fn status_metrics(elapsed: Duration, status: &Status) -> CommandMetrics {
    let total_ms = elapsed.as_millis();
    CommandMetrics {
        command: "status",
        total_ms,
        phases: vec![
            measured_phase("total", total_ms),
            measured_phase("status_load", total_ms),
            unmeasured_phase("state_resolve"),
        ],
        counters: vec![
            counter("open_fires", status.open_fires),
            counter("state_known", usize::from(!status.state.is_empty())),
        ],
        cache: cache_unimplemented(),
    }
}

pub(crate) fn scan_metrics(elapsed: Duration, scan: &codefire_core::ScanResult) -> CommandMetrics {
    let total_ms = elapsed.as_millis();
    CommandMetrics {
        command: "scan",
        total_ms,
        phases: vec![
            measured_phase("total", total_ms),
            measured_phase("scan_pipeline", total_ms),
            unmeasured_phase("atom_extraction"),
            unmeasured_phase("trace_parse"),
            unmeasured_phase("fire_build"),
        ],
        counters: vec![
            counter("atoms", scan.atom_index.atoms.len()),
            counter("trace_links", scan.trace_graph.links.len()),
            counter("changed_atoms", scan.changed_atoms.len()),
            counter("open_fires", scan.open_fires.len()),
        ],
        cache: cache_unimplemented(),
    }
}

pub(crate) fn verification_metrics(
    elapsed: Duration,
    verification: &codefire_core::Verification,
) -> CommandMetrics {
    let total_ms = elapsed.as_millis();
    CommandMetrics {
        command: "verify",
        total_ms,
        phases: vec![
            measured_phase("total", total_ms),
            measured_phase("verification_pipeline", total_ms),
            unmeasured_phase("scan_pipeline"),
            unmeasured_phase("verification_commands"),
            unmeasured_phase("policy_evaluation"),
        ],
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
        cache: cache_unimplemented(),
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
    for phase in &metrics.phases {
        let suffix = if phase.measured { "" } else { " (unmeasured)" };
        println!(
            "  {}: {} ms{}",
            phase_timing_key(phase.name),
            phase.elapsed_ms,
            suffix
        );
    }
    println!("counters:");
    for counter in &metrics.counters {
        println!("  {}: {}", counter.name, counter.value);
    }
    println!("cache:");
    println!("  status: {}", metrics.cache.status);
    println!("  enabled: {}", metrics.cache.enabled);
    println!("  implementation: {}", metrics.cache.implementation);
    println!("  disabled_reason: {}", metrics.cache.disabled_reason);
    println!("  entry_count: {}", metrics.cache.entry_count);
    println!("  hit_count: {}", metrics.cache.hit_count);
    println!("  miss_count: {}", metrics.cache.miss_count);
}

fn metrics_json(metrics: &CommandMetrics) -> Value {
    let mut phase_timings = Map::new();
    for phase in &metrics.phases {
        phase_timings.insert(phase_timing_key(phase.name), json!(phase.elapsed_ms));
    }
    json!({
        "type": "codefire_metrics",
        "version": 1,
        "command": metrics.command,
        "phase_timings": phase_timings,
        "phases": metrics.phases.iter().map(phase_json).collect::<Vec<_>>(),
        "counters": metrics.counters.iter().map(counter_json).collect::<Vec<_>>(),
        "cache": cache_json(&metrics.cache),
    })
}

fn phase_json(phase: &MetricPhase) -> Value {
    json!({
        "name": phase.name,
        "elapsed_ms": phase.elapsed_ms,
        "measured": phase.measured,
    })
}

fn counter_json(counter: &MetricCounter) -> Value {
    json!({
        "name": counter.name,
        "value": counter.value,
    })
}

fn cache_json(cache: &CacheMetrics) -> Value {
    json!({
        "status": cache.status,
        "enabled": cache.enabled,
        "implementation": cache.implementation,
        "disabled_reason": cache.disabled_reason,
        "entry_count": cache.entry_count,
        "hit_count": cache.hit_count,
        "miss_count": cache.miss_count,
    })
}

fn measured_phase(name: &'static str, elapsed_ms: u128) -> MetricPhase {
    MetricPhase {
        name,
        elapsed_ms,
        measured: true,
    }
}

fn unmeasured_phase(name: &'static str) -> MetricPhase {
    MetricPhase {
        name,
        elapsed_ms: 0,
        measured: false,
    }
}

fn counter(name: &'static str, value: usize) -> MetricCounter {
    MetricCounter { name, value }
}

fn cache_unimplemented() -> CacheMetrics {
    CacheMetrics {
        status: "unimplemented",
        enabled: false,
        implementation: "none",
        disabled_reason: "not_implemented",
        entry_count: 0,
        hit_count: 0,
        miss_count: 0,
    }
}

fn phase_timing_key(name: &str) -> String {
    format!("{name}_ms")
}
