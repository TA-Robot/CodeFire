# Operations Runbook Catalog

This catalog expands the operator handbook with concrete, repeatable runbooks for the algorithm evolution lab.
Each entry names the trigger, required inputs, procedure, completion check, and failure handling rule.
Use it when a task is too small for a new design document but still needs deterministic operating guidance.

## Catalog Rules

- Prefer the narrowest runbook that matches the current failure or workflow.
- Keep CodeFire branch state, experiment evidence, and documentation indexes visible during every run.
- Record negative outcomes with the same care as positive outcomes.
- Do not promote a candidate, claim, or documentation batch until the listed completion checks are satisfied.

## R001: Objective Intake

- Trigger: use this runbook when the operator needs to turn a vague research request into bounded experiment work.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `objective-intake` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R002: Scope Control

- Trigger: use this runbook when the operator needs to keep an experiment small enough to run and review.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `scope-control` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R003: Branch Opening

- Trigger: use this runbook when the operator needs to open one CodeFire branch for one coherent documentation or code batch.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `branch-open` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R004: Branch State Check

- Trigger: use this runbook when the operator needs to confirm the open directory and branch head before editing.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `branch-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R005: Trace Review

- Trigger: use this runbook when the operator needs to find missing requirement, design, code, and test links.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `trace-review` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R006: Experiment Plan Creation

- Trigger: use this runbook when the operator needs to convert a candidate idea into a runnable experiment plan.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `experiment-plan` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R007: Artifact Contract Review

- Trigger: use this runbook when the operator needs to check that every run writes the expected files.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `artifact-contract` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R008: Runner Policy Check

- Trigger: use this runbook when the operator needs to avoid unsafe commands and uncontrolled external state.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `runner-policy` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R009: Baseline Choice

- Trigger: use this runbook when the operator needs to select the baseline that matches the benchmark and metric.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `baseline-choice` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R010: Metric Direction Check

- Trigger: use this runbook when the operator needs to avoid interpreting a lower-is-better metric as higher-is-better.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `metric-direction` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R011: Budget Check

- Trigger: use this runbook when the operator needs to confirm cost, batch size, and time limits before execution.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `budget-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R012: Replication Planning

- Trigger: use this runbook when the operator needs to define repeated runs before promotion decisions.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `replication-plan` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R013: Ablation Planning

- Trigger: use this runbook when the operator needs to separate a mechanism test from a broad comparison.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `ablation-plan` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R014: Failure Classification

- Trigger: use this runbook when the operator needs to separate invalid hypothesis, invalid run, timeout, and budget failure.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `failure-classify` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R015: Negative Result Archive

- Trigger: use this runbook when the operator needs to preserve failed paths that should not be retried unchanged.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `negative-archive` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R016: Evidence Pack Assembly

- Trigger: use this runbook when the operator needs to bundle runs, summaries, artifacts, freshness, and objections.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `evidence-pack` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R017: Claim Readiness

- Trigger: use this runbook when the operator needs to decide whether evidence can support a claim or only a hypothesis.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `claim-readiness` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R018: Review Queue Triage

- Trigger: use this runbook when the operator needs to rank claims and objections by blocking effect.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `review-queue` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R019: Release Gate Check

- Trigger: use this runbook when the operator needs to hold external claims until required evidence is present.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `release-gate` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R020: Frontier Ranking

- Trigger: use this runbook when the operator needs to rank research frontier signals by expected value and risk.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `frontier-rank` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R021: Frontier Feedback

- Trigger: use this runbook when the operator needs to feed failed, blocked, and successful results back into priorities.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `frontier-feedback` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R022: Cycle Plan Synthesis

- Trigger: use this runbook when the operator needs to turn current state into active, review, and deferred lanes.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `cycle-plan` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R023: Handoff Bundle Preparation

- Trigger: use this runbook when the operator needs to create a reviewable package for the next operator.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `handoff-bundle` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R024: Manifest Audit

- Trigger: use this runbook when the operator needs to detect drift between listed artifacts and current files.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `manifest-audit` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R025: Stale Evidence Check

- Trigger: use this runbook when the operator needs to detect old benchmark versions and outdated artifacts.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `stale-evidence` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R026: Identifier Naming Audit

- Trigger: use this runbook when the operator needs to keep source identifiers shorter than trace atoms.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `naming-audit` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R027: Documentation Index Update

- Trigger: use this runbook when the operator needs to make new guidance discoverable from entry points.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `documentation-index` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R028: History Update

- Trigger: use this runbook when the operator needs to record completed decisions without rewriting earlier facts.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `history-update` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R029: Todo Update

- Trigger: use this runbook when the operator needs to keep implementation tasks aligned with what actually shipped.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `todo-update` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R030: Roadmap Update

- Trigger: use this runbook when the operator needs to move long-running work forward without hiding unfinished work.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `roadmap-update` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R031: Source Atlas Update

- Trigger: use this runbook when the operator needs to map module changes to ownership and operating guidance.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `source-atlas-update` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R032: Spec Update

- Trigger: use this runbook when the operator needs to add requirements only when behavior or acceptance changes.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `spec-update` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R033: Design Update

- Trigger: use this runbook when the operator needs to capture boundary changes before code grows around them.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `design-update` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R034: Test Selection

- Trigger: use this runbook when the operator needs to run the narrow check first and the full suite before commit.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `test-selection` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R035: Verify Warning Review

- Trigger: use this runbook when the operator needs to distinguish nonblocking trace warnings from release blockers.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `verify-warning` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R036: Merge Dry Run

- Trigger: use this runbook when the operator needs to detect conflicts before mutating the target open directory.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `merge-dry-run` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R037: Main Verification

- Trigger: use this runbook when the operator needs to repeat verification after merge, not only on feature branch.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `main-verify` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R038: Branch Close

- Trigger: use this runbook when the operator needs to remove the feature open directory after main is clean.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `close-branch` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R039: Contamination Check

- Trigger: use this runbook when the operator needs to confirm dataset and artifact overlap risks are recorded.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `contamination-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R040: Overfitting Check

- Trigger: use this runbook when the operator needs to hold work that overuses target validation feedback.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `overfit-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R041: Leaderboard Submission Check

- Trigger: use this runbook when the operator needs to avoid excessive probing of external leaderboards.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `leaderboard-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R042: Human Override Review

- Trigger: use this runbook when the operator needs to record rationale whenever automation is bypassed.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `override-review` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R043: Risk Register Update

- Trigger: use this runbook when the operator needs to rank open risks by probability and blocking effect.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `risk-register` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R044: Limitation Section Drafting

- Trigger: use this runbook when the operator needs to state known gaps before promotion or release.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `limitation-section` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R045: Report Bundle Build

- Trigger: use this runbook when the operator needs to collect artifacts needed for a reviewable research report.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `report-bundle` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R046: Reproduction Checklist

- Trigger: use this runbook when the operator needs to confirm repeated artifacts before stronger evidence claims.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `reproduction-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R047: Learning Curve Review

- Trigger: use this runbook when the operator needs to detect plateaus and instability before continuing spend.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `learning-curve` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R048: Regression Check

- Trigger: use this runbook when the operator needs to catch shared benchmark drops introduced by a change.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `regression-check` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R049: Policy Evaluation

- Trigger: use this runbook when the operator needs to measure whether selection policy improved outcomes.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `policy-eval` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R050: Memory Index Update

- Trigger: use this runbook when the operator needs to make durable records searchable by tag, kind, and text.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `memory-index` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R051: Run Identity Ledger

- Trigger: use this runbook when the operator needs to link run logs, artifacts, evidence, and candidate identity.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `run-identity` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R052: Configuration Capture

- Trigger: use this runbook when the operator needs to record settings needed to reproduce a run.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `configuration-capture` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R053: Compute Accounting

- Trigger: use this runbook when the operator needs to charge failed attempts and overrun attempts to budget.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `compute-accounting` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R054: Novelty Review

- Trigger: use this runbook when the operator needs to flag known family overlap before novelty claims.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `novelty-review` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R055: External Validity Review

- Trigger: use this runbook when the operator needs to check transfer across domains before broad claims.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `external-validity` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R056: Stop Criteria

- Trigger: use this runbook when the operator needs to decide when additional work is no longer justified.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `stop-criteria` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R057: Retry Escalation

- Trigger: use this runbook when the operator needs to route recoverable failures to rerun, revise, defer, or archive.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `retry-escalation` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R058: Progress Meter Review

- Trigger: use this runbook when the operator needs to measure whether the program is accumulating useful evidence.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `progress-meter` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R059: Role Registry Check

- Trigger: use this runbook when the operator needs to confirm new components have a clear role and owner.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `role-registry` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R060: Bundle Readiness Gate

- Trigger: use this runbook when the operator needs to block handoff packages with drift or missing review material.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `bundle-readiness` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R061: Plan Lint

- Trigger: use this runbook when the operator needs to catch malformed cycle plans before they become operating input.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `plan-lint` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R062: Summary Manifest Review

- Trigger: use this runbook when the operator needs to verify that summary artifacts match the current archive.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `summary-manifest` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R063: Audit Trail Review

- Trigger: use this runbook when the operator needs to preserve decision history for claims and overrides.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `audit-trail` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R064: Documentation Count Audit

- Trigger: use this runbook when the operator needs to measure line and word counts against the doubling target.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `docs-count` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R065: Documentation Staleness Review

- Trigger: use this runbook when the operator needs to find guidance that no longer matches source behavior.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `docs-staleness` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R066: Documentation Crosslink Review

- Trigger: use this runbook when the operator needs to ensure each long-lived guide is reachable from an index.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `docs-crosslink` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R067: Documentation Example Review

- Trigger: use this runbook when the operator needs to keep examples clearly marked when they are not real results.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `docs-example` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

## R068: Documentation Closeout

- Trigger: use this runbook when the operator needs to finish a docs batch with status, verification, and history.
- Inputs: current objective, active CodeFire branch state, relevant source or documentation files, and the latest experiment evidence.
- Output: a bounded decision, updated artifact, or explicit blocker that another operator can inspect without replaying hidden context.
- Precheck: confirm the working directory belongs to the intended branch and that unrelated user changes are not being overwritten.
- Step 1: state the smallest useful outcome for `docs-closeout` and write down what evidence would prove it is complete.
- Step 2: inspect the authoritative files or command output before trusting memory from a previous run.
- Step 3: apply the smallest coherent edit, run, or review action that moves the evidence state forward.
- Step 4: update the nearest index, history entry, experiment note, or todo item if the work creates durable knowledge.
- Step 5: run the narrow verification first, then the broader verification required by the repository instructions.
- Completion check: the next operator can identify the changed artifact, the reason for the change, and the verification result.
- Failure handling: if the check fails, keep the failure visible, classify it, and choose rerun, revise, defer, or archive-negative.
- Documentation note: update operations guidance only when the workflow would otherwise need to be rediscovered from source or tests.
- CodeFire note: do not close the branch while pending changes, open fires, or merge conflicts remain unresolved.

