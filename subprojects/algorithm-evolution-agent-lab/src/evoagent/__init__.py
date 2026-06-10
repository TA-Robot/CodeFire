"""Experiment automation primitives for algorithm evolution."""

from evoagent.loop import EvolutionAgent
from evoagent.analysis import AnalysisNote, RunAnalyst
from evoagent.automation_surface import (
    ExperimentAutomationSurfaceGuard,
    ProductSurface,
    ProductWorkItem,
    SurfacePriorityFinding,
)
from evoagent.automation_policy import AutomationPolicy, AutomationPolicyFinding, SafeAutomationPolicy
from evoagent.artifacts import ArtifactCollectionContract, ArtifactCollectionReport, DeclaredArtifact
from evoagent.baselines import BaselineRecord, BaselineRegistry
from evoagent.baseline_reproduction import BaselineReproduction, BaselineReproductionLedger
from evoagent.batch_planning import BudgetedExperimentBatchPlanner, ExperimentBatch
from evoagent.benchmark_domains import BenchmarkDomainCatalog, BenchmarkDomainDefinition
from evoagent.benchmarks import (
    BenchmarkDefinition,
    BenchmarkMetric,
    BenchmarkRegistry as BenchmarkDefinitionRegistry,
    RepeatedRunSummary,
    summarize_repeated_runs,
)
from evoagent.budgeting import BudgetAwarePlanner, BudgetPlan
from evoagent.campaign import CampaignMilestone, CampaignProgress, ExperimentCampaignTracker
from evoagent.challenge import ChallengeBaseline, ComputeBudget, SotaChallengeDefinition, load_challenge_definition
from evoagent.claim_audit import ClaimAuditEvent, ClaimAuditStage, ClaimAuditTrail, ClaimAuditTrailBuilder
from evoagent.claim_release import (
    ClaimReleaseDecision,
    ClaimReleaseGate,
    ClaimReleaseGateResult,
    ClaimReleaseInput,
)
from evoagent.claim_review_queue import (
    ClaimReviewDecision,
    ClaimReviewItem,
    ClaimReviewQueue,
    ClaimReviewRequest,
)
from evoagent.claims import ClaimDisciplineClassifier, ClaimDisciplineResult, ClaimType
from evoagent.compute import ComputeAccount, ComputeAccountant
from evoagent.configuration import ConfigurationCapture, ConfigurationFinding, ExperimentConfiguration
from evoagent.contamination import ContaminationFinding, ContaminationSource, ContaminationSourceRegistry
from evoagent.decomposition import ObjectiveDecomposer, ResearchSubgoal
from evoagent.dependencies import ExperimentDependency, ExperimentDependencyGraph
from evoagent.models import (
    CandidateAlgorithm,
    ExperimentGoal,
    ExperimentPlan,
    ExperimentResult,
    ExperimentRun,
    Hypothesis,
    HypothesisLink,
    RunArtifact,
    stable_id,
)
from evoagent.overfitting import AntiOverfittingMonitor, OverfittingFinding, ProbeRecord
from evoagent.plan_contract import ExperimentPlanContract, ExperimentPlanContractBuilder, PlanContractFinding
from evoagent.planner import CodexPlanner, Planner, StaticPlanner
from evoagent.policy_eval import PolicyEvaluationReport, PolicyEvaluator
from evoagent.evidence import EvidenceLedger, EvidenceRecord
from evoagent.evolution import AblationAxis, AblationGenerator, CandidateEvolutionOperators, MutationSpec
from evoagent.evidence_pack import (
    EvidenceReadiness,
    ExperimentEvidencePack,
    ExperimentEvidencePackBuilder,
    ExperimentEvidenceSignal,
)
from evoagent.evidence_freshness import EvidenceFreshnessAudit, EvidenceFreshnessFinding, VersionedEvidenceRef
from evoagent.external_validity import BenchmarkDomain, ExternalValidityEvaluator, ExternalValidityReport
from evoagent.failures import FailureClassification, FailureClassifier
from evoagent.fixtures import FixtureResult, ToyTabularFixture
from evoagent.ingestion import ResultIngestor
from evoagent.iteration import ExperimentIterationPlan, ExperimentIterationPlanner, IterationLaneItem
from evoagent.leakage import DatasetSplitAudit, LeakageChecker, LeakageFinding
from evoagent.learning_curve import LearningCurveAnalyzer, LearningCurveReport
from evoagent.leaderboard import LeaderboardSubmission, LeaderboardSubmissionFinding, LeaderboardSubmissionLedger
from evoagent.limitations import Limitation, LimitationSectionGenerator
from evoagent.memory import MemoryRecord, ResearchMemoryIndex
from evoagent.metrics import MetricNormalizer, NormalizedMetricResult
from evoagent.negative_results import NegativeResult, NegativeResultArchive
from evoagent.novelty import AlgorithmFamilyReference, NoveltyFinding, NoveltyReviewer
from evoagent.overrides import HumanOverride, HumanOverrideLedger
from evoagent.program import DecisionRecord, DecisionRecordFinding, DecisionRecordValidator, ResearchProgram
from evoagent.progress import ResearchProgressMeter, ResearchProgressReport
from evoagent.promotion import (
    CandidatePromotionInput,
    CandidatePromotionPolicy,
    CandidatePromotionResult,
    PromotionDecision,
)
from evoagent.queueing import ExperimentQueue, QueuedExperiment
from evoagent.report_bundle import SotaReportBundleGenerator, SotaReportBundleManifest
from evoagent.reporting import LivingResearchReport, ReportOptions
from evoagent.reproduction import ReproductionChecklist, ReproductionChecklistBuilder, ReproductionChecklistItem
from evoagent.regression import RegressionDetector, RegressionFinding
from evoagent.risk import ExperimentRisk, ExperimentRiskRegister, RiskPriority, RiskSeverity, RiskStatus
from evoagent.retrospective import (
    ResearchCyclePlanningHandoffArtifact,
    ResearchCyclePlanningHandoffBundle,
    ResearchCyclePlanningHandoffBundleBuilder,
    ResearchCyclePlan,
    ResearchCyclePlanItem,
    ResearchCyclePlanLane,
    ResearchCyclePlanningPacket,
    ResearchCyclePlanningPacketBuilder,
    ResearchCyclePlanningPacketManifest,
    ResearchCyclePlanningPacketManifestBuilder,
    ResearchCyclePlanningPacketManifestEntry,
    ResearchCyclePlanningPacketManifestMarkdown,
    ResearchCyclePlanningPacketManifestVerification,
    ResearchCyclePlanningPacketManifestVerificationFinding,
    ResearchCyclePlanningPacketManifestVerificationMarkdown,
    ResearchCyclePlanningPacketManifestVerifier,
    ResearchCyclePlanningPacketMarkdown,
    ResearchCyclePlanLint,
    ResearchCyclePlanLintFinding,
    ResearchCyclePlanLintMarkdown,
    ResearchCyclePlanLintReport,
    ResearchCyclePlanLintSeverity,
    ResearchCyclePlanMarkdown,
    ResearchCyclePlanSynthesizer,
    ResearchCycleRetrospective,
    ResearchCycleRetrospectiveReport,
    ResearchCycleSignal,
    RetrospectivePlanningSummary,
    RetrospectivePriority,
    RetrospectiveRecommendation,
)
from evoagent.review import ReviewLedger, ReviewObjection
from evoagent.roles import RoleComponent, RoleRegistry
from evoagent.run_identity import RunIdentityLedger, RunIdentityRecord
from evoagent.runner import LocalExperimentRunner
from evoagent.retry_planning import RetryDecision, RetryEscalationPlanner, RetryRecommendation
from evoagent.safety import ClaimEvidenceSummary, ClaimReadiness, can_claim_sota, claim_readiness
from evoagent.scheduler import PortfolioPolicy, PortfolioScheduler
from evoagent.scoring import score_candidate
from evoagent.sota_framing import SotaPursuitFrame, SotaPursuitFramer
from evoagent.statistics import ConfidenceInterval, ConfidenceIntervalEstimator
from evoagent.stop_criteria import StopCriteria, StopCriteriaEvaluator, StopFinding, StopObservation
from evoagent.traceability import CodeFireTraceAudit, TraceAuditFinding, TraceAuditReport, TraceLink
from evoagent.triage import ExperimentTriageBoard, ExperimentTriageItem, ExperimentTriageSignal, TriageAction

__all__ = [
    "CandidateAlgorithm",
    "CandidateEvolutionOperators",
    "CandidatePromotionInput",
    "CandidatePromotionPolicy",
    "CandidatePromotionResult",
    "ChallengeBaseline",
    "ClaimEvidenceSummary",
    "ClaimAuditEvent",
    "ClaimAuditStage",
    "ClaimAuditTrail",
    "ClaimAuditTrailBuilder",
    "ClaimReleaseDecision",
    "ClaimReleaseGate",
    "ClaimReleaseGateResult",
    "ClaimReleaseInput",
    "ClaimDisciplineClassifier",
    "ClaimDisciplineResult",
    "ClaimReviewDecision",
    "ClaimReviewItem",
    "ClaimReviewQueue",
    "ClaimReviewRequest",
    "ClaimReadiness",
    "ClaimType",
    "CodeFireTraceAudit",
    "ComputeAccount",
    "ComputeAccountant",
    "ComputeBudget",
    "ContaminationFinding",
    "ContaminationSource",
    "ContaminationSourceRegistry",
    "ConfigurationCapture",
    "ConfigurationFinding",
    "ConfidenceInterval",
    "ConfidenceIntervalEstimator",
    "DatasetSplitAudit",
    "ObjectiveDecomposer",
    "BaselineRecord",
    "BaselineReproduction",
    "BaselineReproductionLedger",
    "BaselineRegistry",
    "BudgetedExperimentBatchPlanner",
    "BenchmarkDomain",
    "BenchmarkDomainCatalog",
    "BenchmarkDomainDefinition",
    "BenchmarkDefinition",
    "BenchmarkDefinitionRegistry",
    "BenchmarkMetric",
    "BudgetAwarePlanner",
    "BudgetPlan",
    "CampaignMilestone",
    "CampaignProgress",
    "AnalysisNote",
    "AblationAxis",
    "AblationGenerator",
    "AlgorithmFamilyReference",
    "AntiOverfittingMonitor",
    "ArtifactCollectionContract",
    "ArtifactCollectionReport",
    "AutomationPolicy",
    "AutomationPolicyFinding",
    "EvolutionAgent",
    "ExperimentGoal",
    "ExperimentIterationPlan",
    "ExperimentIterationPlanner",
    "ExperimentConfiguration",
    "ExperimentAutomationSurfaceGuard",
    "ExperimentBatch",
    "ExperimentCampaignTracker",
    "ExperimentDependency",
    "ExperimentDependencyGraph",
    "ExperimentEvidencePack",
    "ExperimentEvidencePackBuilder",
    "ExperimentEvidenceSignal",
    "ExperimentPlan",
    "ExperimentPlanContract",
    "ExperimentPlanContractBuilder",
    "ExperimentQueue",
    "ExperimentRisk",
    "ExperimentRiskRegister",
    "ExperimentResult",
    "ExperimentRun",
    "ExperimentTriageBoard",
    "ExperimentTriageItem",
    "ExperimentTriageSignal",
    "EvidenceLedger",
    "EvidenceRecord",
    "EvidenceReadiness",
    "EvidenceFreshnessAudit",
    "EvidenceFreshnessFinding",
    "ExternalValidityEvaluator",
    "ExternalValidityReport",
    "FailureClassification",
    "FailureClassifier",
    "FixtureResult",
    "Hypothesis",
    "HypothesisLink",
    "IterationLaneItem",
    "HumanOverride",
    "HumanOverrideLedger",
    "LocalExperimentRunner",
    "LeakageChecker",
    "LeakageFinding",
    "LeaderboardSubmission",
    "LeaderboardSubmissionFinding",
    "LeaderboardSubmissionLedger",
    "LearningCurveAnalyzer",
    "LearningCurveReport",
    "Limitation",
    "LimitationSectionGenerator",
    "LivingResearchReport",
    "MemoryRecord",
    "MetricNormalizer",
    "MutationSpec",
    "NegativeResult",
    "NegativeResultArchive",
    "NormalizedMetricResult",
    "NoveltyFinding",
    "NoveltyReviewer",
    "CodexPlanner",
    "DecisionRecord",
    "DecisionRecordFinding",
    "DecisionRecordValidator",
    "DeclaredArtifact",
    "Planner",
    "PlanContractFinding",
    "ProbeRecord",
    "PolicyEvaluationReport",
    "PolicyEvaluator",
    "PortfolioPolicy",
    "PortfolioScheduler",
    "ProductSurface",
    "ProductWorkItem",
    "PromotionDecision",
    "QueuedExperiment",
    "OverfittingFinding",
    "ResearchProgram",
    "ResearchMemoryIndex",
    "ResearchProgressMeter",
    "ResearchProgressReport",
    "ResearchCyclePlanningHandoffArtifact",
    "ResearchCyclePlanningHandoffBundle",
    "ResearchCyclePlanningHandoffBundleBuilder",
    "ResearchCyclePlan",
    "ResearchCyclePlanItem",
    "ResearchCyclePlanLane",
    "ResearchCyclePlanningPacket",
    "ResearchCyclePlanningPacketBuilder",
    "ResearchCyclePlanningPacketManifest",
    "ResearchCyclePlanningPacketManifestBuilder",
    "ResearchCyclePlanningPacketManifestEntry",
    "ResearchCyclePlanningPacketManifestMarkdown",
    "ResearchCyclePlanningPacketManifestVerification",
    "ResearchCyclePlanningPacketManifestVerificationFinding",
    "ResearchCyclePlanningPacketManifestVerificationMarkdown",
    "ResearchCyclePlanningPacketManifestVerifier",
    "ResearchCyclePlanningPacketMarkdown",
    "ResearchCyclePlanLint",
    "ResearchCyclePlanLintFinding",
    "ResearchCyclePlanLintMarkdown",
    "ResearchCyclePlanLintReport",
    "ResearchCyclePlanLintSeverity",
    "ResearchCyclePlanMarkdown",
    "ResearchCyclePlanSynthesizer",
    "ResearchCycleRetrospective",
    "ResearchCycleRetrospectiveReport",
    "ResearchCycleSignal",
    "ResearchSubgoal",
    "ReportOptions",
    "ReproductionChecklist",
    "ReproductionChecklistBuilder",
    "ReproductionChecklistItem",
    "RegressionDetector",
    "RegressionFinding",
    "ResultIngestor",
    "RiskPriority",
    "RiskSeverity",
    "RiskStatus",
    "ReviewLedger",
    "ReviewObjection",
    "RetryDecision",
    "RetryEscalationPlanner",
    "RetryRecommendation",
    "RoleComponent",
    "RoleRegistry",
    "RunArtifact",
    "RunAnalyst",
    "RunIdentityLedger",
    "RunIdentityRecord",
    "SafeAutomationPolicy",
    "StaticPlanner",
    "SotaChallengeDefinition",
    "SotaReportBundleGenerator",
    "SotaReportBundleManifest",
    "SotaPursuitFrame",
    "SotaPursuitFramer",
    "StopCriteria",
    "StopCriteriaEvaluator",
    "StopFinding",
    "StopObservation",
    "ToyTabularFixture",
    "TraceAuditFinding",
    "TraceAuditReport",
    "TraceLink",
    "TriageAction",
    "SurfacePriorityFinding",
    "VersionedEvidenceRef",
    "RepeatedRunSummary",
    "RetrospectivePlanningSummary",
    "RetrospectivePriority",
    "RetrospectiveRecommendation",
    "can_claim_sota",
    "claim_readiness",
    "load_challenge_definition",
    "score_candidate",
    "stable_id",
    "summarize_repeated_runs",
]
