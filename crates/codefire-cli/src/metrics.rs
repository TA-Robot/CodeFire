use crate::{branch_refs::BranchRefWork, Branch, Status};
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::io::{self, Write};
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct CommandMetrics {
    command: &'static str,
    total_ms: u128,
    phases: Vec<MetricPhase>,
    counters: Vec<MetricCounter>,
    cache: CacheMetrics,
}

#[derive(Debug, Clone, Serialize)]
struct MetricPhase {
    name: &'static str,
    #[serde(skip)]
    timing_key: &'static str,
    elapsed_ms: u128,
    measured: bool,
}

#[derive(Debug, Clone, Serialize)]
struct MetricCounter {
    name: &'static str,
    value: u64,
}

#[derive(Debug, Clone, Serialize)]
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

pub(crate) fn branch_list_metrics(
    elapsed: Duration,
    branches: &[Branch],
    store_work: codefire_store::SealedValidationWork,
    ref_work: BranchRefWork,
    decision_work: &codefire_core::DecisionWorkSnapshot,
) -> CommandMetrics {
    let total_ms = elapsed.as_millis();
    CommandMetrics {
        command: "branch-list",
        total_ms,
        phases: vec![
            measured_phase("total", total_ms),
            measured_phase("branch_list_load", total_ms),
        ],
        counters: vec![
            counter("branches", branches.len()),
            counter(
                "open_branches",
                branches
                    .iter()
                    .filter(|branch| branch.state.starts_with("open-"))
                    .count(),
            ),
            counter_u64(
                "sealed_semantic_validations",
                store_work.semantic_commit_validations,
            ),
            counter_u64(
                "sealed_validated_commit_hits",
                store_work.validated_commit_hits,
            ),
            counter_u64("sealed_document_reads", store_work.document_reads),
            counter_u64("sealed_document_read_bytes", store_work.document_read_bytes),
            counter_u64("sealed_document_memo_hits", store_work.document_memo_hits),
            counter_u64(
                "sealed_snapshot_revalidation_records",
                store_work.revalidated_documents,
            ),
            counter_u64(
                "sealed_snapshot_revalidation_bytes",
                store_work.revalidated_bytes,
            ),
            counter_u64("branch_ref_records", ref_work.records),
            counter_u64("branch_ref_bytes", ref_work.bytes),
            counter_u64(
                "branch_ref_snapshot_revalidation_records",
                ref_work.revalidated_records,
            ),
            counter_u64(
                "branch_ref_snapshot_revalidation_bytes",
                ref_work.revalidated_bytes,
            ),
            counter_u64(
                "decision_graph_nodes",
                cumulative_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::GraphNodes,
                ),
            ),
            counter_u64(
                "decision_graph_edges",
                cumulative_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::GraphEdges,
                ),
            ),
            counter_u64(
                "decision_logical_depth",
                maximum_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::LogicalDepth,
                ),
            ),
            counter_u64(
                "decision_maximum_frontier",
                maximum_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::MaximumFrontier,
                ),
            ),
            counter_u64(
                "decision_object_records",
                cumulative_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::ObjectRecords,
                ),
            ),
            counter_u64(
                "decision_aggregate_bytes",
                cumulative_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                ),
            ),
            counter_u64(
                "decision_files_read",
                decision_counter(decision_work, codefire_core::DecisionWorkCounter::FilesRead),
            ),
            counter_u64(
                "decision_bytes_hashed",
                decision_counter(
                    decision_work,
                    codefire_core::DecisionWorkCounter::BytesHashed,
                ),
            ),
            counter_u64(
                "decision_snapshot_revalidation_records",
                cumulative_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
                ),
            ),
            counter_u64(
                "decision_snapshot_revalidation_bytes",
                cumulative_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
                ),
            ),
            counter_u64(
                "decision_retained_bytes_peak",
                peak_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                ),
            ),
            counter_u64(
                "decision_temporary_bytes_peak",
                peak_work(
                    decision_work,
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                ),
            ),
        ],
        cache: CacheMetrics {
            status: "aggregate",
            enabled: true,
            implementation: "sealed_validation_session_v1",
            disabled_reason: "",
            entry_count: saturating_usize(store_work.document_reads),
            hit_count: saturating_usize(
                store_work
                    .validated_commit_hits
                    .saturating_add(store_work.document_memo_hits),
            ),
            miss_count: saturating_usize(store_work.document_reads),
        },
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
    let mut stdout = io::stdout().lock();
    write_metrics(&mut stdout, metrics).expect("writing metrics to stdout failed");
}

pub(crate) fn write_metrics<W: Write + ?Sized>(
    writer: &mut W,
    metrics: &CommandMetrics,
) -> io::Result<()> {
    writeln!(writer, "CodeFire metrics")?;
    writeln!(writer, "command: {}", metrics.command)?;
    writeln!(writer, "total: {} ms", metrics.total_ms)?;
    writeln!(writer, "phase timings:")?;
    for phase in &metrics.phases {
        let suffix = if phase.measured { "" } else { " (unmeasured)" };
        writeln!(
            writer,
            "  {}: {} ms{}",
            phase.timing_key, phase.elapsed_ms, suffix
        )?;
    }
    writeln!(writer, "counters:")?;
    for counter in &metrics.counters {
        writeln!(writer, "  {}: {}", counter.name, counter.value)?;
    }
    writeln!(writer, "cache:")?;
    writeln!(writer, "  status: {}", metrics.cache.status)?;
    writeln!(writer, "  enabled: {}", metrics.cache.enabled)?;
    writeln!(writer, "  implementation: {}", metrics.cache.implementation)?;
    writeln!(
        writer,
        "  disabled_reason: {}",
        metrics.cache.disabled_reason
    )?;
    writeln!(writer, "  entry_count: {}", metrics.cache.entry_count)?;
    writeln!(writer, "  hit_count: {}", metrics.cache.hit_count)?;
    writeln!(writer, "  miss_count: {}", metrics.cache.miss_count)?;
    Ok(())
}

pub(crate) struct MetricsWire<'a>(&'a CommandMetrics);

pub(crate) fn metrics_wire(metrics: &CommandMetrics) -> MetricsWire<'_> {
    MetricsWire(metrics)
}

struct PhaseTimings<'a>(&'a [MetricPhase]);

impl Serialize for PhaseTimings<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for phase in self.0 {
            if phase.measured {
                map.serialize_entry(phase.timing_key, &phase.elapsed_ms)?;
            } else {
                map.serialize_entry(phase.timing_key, &Option::<u128>::None)?;
            }
        }
        map.end()
    }
}

impl Serialize for MetricsWire<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let metrics = self.0;
        let mut map = serializer.serialize_map(Some(7))?;
        map.serialize_entry("type", "codefire_metrics")?;
        map.serialize_entry("version", &1_u64)?;
        map.serialize_entry("command", metrics.command)?;
        map.serialize_entry("phase_timings", &PhaseTimings(&metrics.phases))?;
        map.serialize_entry("phases", &metrics.phases)?;
        map.serialize_entry("counters", &metrics.counters)?;
        if metrics.cache.status == "unimplemented" {
            map.serialize_entry("cache_status", "unavailable")?;
        } else {
            map.serialize_entry("cache", &metrics.cache)?;
        }
        map.end()
    }
}

fn metrics_json(metrics: &CommandMetrics) -> Value {
    let mut phase_timings = Map::new();
    for phase in &metrics.phases {
        let value = if phase.measured {
            json!(phase.elapsed_ms)
        } else {
            Value::Null
        };
        phase_timings.insert(phase.timing_key.to_string(), value);
    }
    let mut value = json!({
        "type": "codefire_metrics",
        "version": 1,
        "command": metrics.command,
        "phase_timings": phase_timings,
        "phases": metrics.phases.iter().map(phase_json).collect::<Vec<_>>(),
        "counters": metrics.counters.iter().map(counter_json).collect::<Vec<_>>(),
    });
    if metrics.cache.status == "unimplemented" {
        value["cache_status"] = json!("unavailable");
    } else {
        value["cache"] = cache_json(&metrics.cache);
    }
    value
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
        timing_key: phase_timing_key(name),
        elapsed_ms,
        measured: true,
    }
}

fn unmeasured_phase(name: &'static str) -> MetricPhase {
    MetricPhase {
        name,
        timing_key: phase_timing_key(name),
        elapsed_ms: 0,
        measured: false,
    }
}

fn counter(name: &'static str, value: usize) -> MetricCounter {
    MetricCounter {
        name,
        value: u64::try_from(value).unwrap_or(u64::MAX),
    }
}

fn counter_u64(name: &'static str, value: u64) -> MetricCounter {
    MetricCounter { name, value }
}

fn cumulative_work(
    snapshot: &codefire_core::DecisionWorkSnapshot,
    dimension: codefire_core::DecisionWorkDimension,
) -> u64 {
    snapshot
        .cumulative
        .get(&dimension)
        .copied()
        .unwrap_or_default()
}

fn maximum_work(
    snapshot: &codefire_core::DecisionWorkSnapshot,
    dimension: codefire_core::DecisionWorkDimension,
) -> u64 {
    snapshot
        .maximum
        .get(&dimension)
        .copied()
        .unwrap_or_default()
}

fn peak_work(
    snapshot: &codefire_core::DecisionWorkSnapshot,
    dimension: codefire_core::DecisionWorkDimension,
) -> u64 {
    snapshot.peak.get(&dimension).copied().unwrap_or_default()
}

fn decision_counter(
    snapshot: &codefire_core::DecisionWorkSnapshot,
    counter: codefire_core::DecisionWorkCounter,
) -> u64 {
    snapshot.counters.get(&counter).copied().unwrap_or_default()
}

fn saturating_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
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

fn phase_timing_key(name: &str) -> &'static str {
    match name {
        "total" => "total_ms",
        "status_load" => "status_load_ms",
        "state_resolve" => "state_resolve_ms",
        "branch_list_load" => "branch_list_load_ms",
        "scan_pipeline" => "scan_pipeline_ms",
        "atom_extraction" => "atom_extraction_ms",
        "trace_parse" => "trace_parse_ms",
        "fire_build" => "fire_build_ms",
        "verification_pipeline" => "verification_pipeline_ms",
        "verification_commands" => "verification_commands_ms",
        "policy_evaluation" => "policy_evaluation_ms",
        _ => "unknown_phase_ms",
    }
}
