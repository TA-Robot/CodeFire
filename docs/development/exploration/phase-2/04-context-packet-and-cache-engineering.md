# Context, Packet, And Cache Engineering

> **非正本 / exploration only**
>
> これはContext Pack、Task Packet、Review Packet、handoff、prompt cache、long-running contextを検討する研究ノートである。採用された結論はdocs/development/ai/、JSON Schema、packet generator、context command、eval suiteへpromotionする。

## 1. Current Shape

observed:

- context-packs.jsonはrequired_files、likely_files、extends、purposeを持つ。
- operating-protocol.mdはTask Packet、Review Packet、handoff shapeを人間向けMarkdownで定義する。
- task-packets/には具体的なready packetがある。
- role-routing-matrix.jsonはrole、reasoning tier、outputを機械可読化している。
- checkerはrequired docs、JSON shape、path reference、task section、stale phraseを検査する。

未解決:

- packet identityとschema validation
- base revision、context freshness、digest binding
- file/Atom/command/invariant/testを横断したselection
- token estimateとcache strategy
- sensitive context、prompt injection、redaction
- reviewしたpatchと統合patchの同一性
- packet migrationとgenerated Markdown

## 2. Design Principle

contextは大量に与えるほど安全になるとは限らない。

~~~text
Useful context =
  relevant
  + authoritative
  + fresh
  + bounded
  + attributable
  + explicit omissions
~~~

Long contextはretrieval failureを隠し、low-signal history、stale docs、conflicting instructionsを増やす可能性がある。

HYP-AI-040:

hypothesis: full repository readingより、graph-derived context plus explicit coverage/omission manifestの方が、少ないtokenでscope/invariant missを減らす。

反証条件:

- graph/source mapがstaleでcritical fileを恒常的に漏らす。
- context generation latencyがmanual readより高い。
- reviewerがfull repo探索を毎回やり直す。

対応実験: EXP-AI-002。

## 3. Canonical Context Pack Candidate

~~~json
{
  "schema": "codefire.context_pack.v1",
  "id": "ctx-example",
  "subject": {
    "kind": "task",
    "id": "task-example"
  },
  "base": {
    "revision": "base-id",
    "treeDigest": "sha256"
  },
  "budget": {
    "maxTokens": 60000,
    "estimatedTokens": 42000
  },
  "entries": [
    {
      "uri": "repo://docs/01_product_definition.md",
      "digest": "sha256",
      "authority": "product-requirement",
      "selector": {
        "kind": "line-or-atom",
        "value": "FR-017"
      },
      "reason": "defines context command acceptance",
      "freshness": "base-bound",
      "sensitivity": "repository"
    }
  ],
  "omissions": [
    {
      "pattern": "target/**",
      "reason": "generated build output"
    }
  ],
  "coverage": {
    "commands": ["context"],
    "invariants": ["path safety", "JSON envelope"],
    "tests": ["focused-test-id"]
  },
  "generatedBy": {
    "tool": "codefire-context-builder",
    "version": "candidate"
  }
}
~~~

Required properties:

- stable ID and schema
- base revision/tree digest
- entry URI and content digest
- authority class
- selection reason
- freshness
- budget estimate
- omissions
- coverage claims
- sensitivity/redaction
- generator identity

context pack自体が正しさを証明するのではなく、reviewerがcoverage claimをchallenge可能にする。

## 4. Context Selection Graph

candidate inputs:

- task objective and acceptance
- allowed/disallowed paths
- command registry
- source-code map and command trace
- module ownership
- persistence/object references
- product requirements and ADR
- changed files/Atoms
- Trace Graph neighbors
- historical defects touching same invariant
- focused tests and validation commands
- transitive packet dependencies

Candidate ranking:

~~~text
score(entry) =
  authority
  * direct_dependency
  * change_proximity
  * invariant_relevance
  * historical_failure_signal
  * freshness
  / token_cost
~~~

scoreはmachine truthではなくheuristicである。required authority sourcesはrankingで落とさない。

Q-CTX-001:

context selectorの正本をstatic manifest、source graph、runtime discoveryのどこへ置くか。

candidate answer:

- static manifest: universal/role invariants
- source graph: task-specific candidate
- runtime discovery: changed state and environment
- packet: final selected/omitted snapshot

## 5. Task Packet Candidate

~~~json
{
  "schema": "codefire.task_packet.v1",
  "id": "task-example",
  "status": "ready",
  "objective": "one verifiable outcome",
  "authority": {
    "request": "issue-or-user-reference"
  },
  "base": {
    "revision": "base-id",
    "treeDigest": "sha256"
  },
  "scope": {
    "readSet": [],
    "writeSet": [],
    "disallowed": []
  },
  "nonGoals": [],
  "invariants": [],
  "acceptanceCriteria": [
    {
      "id": "AC-1",
      "assertion": "observable result",
      "evidenceRequired": ["test"]
    }
  ],
  "validation": {
    "required": [],
    "optional": [],
    "unavailablePolicy": "block-or-explicit-risk"
  },
  "risk": {
    "score": 0,
    "tier": "medium",
    "reasons": []
  },
  "dependencies": [],
  "contextPackDigest": "sha256",
  "handoffSchema": "codefire.handoff.v1"
}
~~~

Fields omitted from current Markdown packet but important:

- packet ID and schema
- authority/request reference
- base/tree digest
- explicit read/write set
- acceptance IDs
- evidence class per assertion
- dependency packet digests
- risk route and reasons
- unavailable gate policy
- supersedes/supersededBy

Task Packetのcanonical representationをJSONにし、Markdownをrendererで生成するcandidateを検討する。human-first authoringを残す場合もround-tripとschema lintが必要。

## 6. Review Packet Candidate

~~~json
{
  "schema": "codefire.review_packet.v1",
  "id": "review-example",
  "taskPacketDigest": "sha256",
  "target": {
    "baseDigest": "sha256",
    "patchDigest": "sha256",
    "headDigest": "sha256"
  },
  "reviewer": {
    "role": "reviewer",
    "independentFromImplementation": true
  },
  "claims": [
    {
      "acceptanceId": "AC-1",
      "decision": "supported",
      "evidenceRefs": []
    }
  ],
  "invariantAssessment": [],
  "validationAssessment": [],
  "decision": "approve",
  "residualRisks": []
}
~~~

Invariant:

review target patch digestとintegration candidate patch digestが異なる場合、approvalはstaleになる。

HYP-AI-041:

hypothesis: task/patch/review digest bindingにより、review後のsilent editとwrong-base approvalをほぼ機械的に排除できる。

反証条件:

- formatter/generated filesでdigestが頻繁に変わり、meaning-preserving changesでもreviewが無効になる。
- large binary/checkoutでdigest計算costが許容できない。

対応:

- raw patch digestとnormalized semantic manifest digestを分けて実験する。
- any mutation後に少なくともchanged subset reviewを要求する。

## 7. Handoff Candidate

handoffは会話summaryではなく、次laneへ渡すtyped artifactとする。

Required:

- role、task/packet ID
- base/head/patch digest
- changed and intentionally untouched paths
- design summary
- acceptance/evidence table
- exact validation commands and results
- skipped/failed/unavailable gates
- risks and follow-up
- effective model/runtime/effort
- duration/tool-call summary

agentのinternal chain-of-thoughtやhidden reasoningは要求しない。保存するのは外部検証可能なdecision rationaleとevidenceだけ。

## 8. Prompt Construction And Cache

documented: OpenAI Prompt Cachingはexact prefix matchを必要とし、static instructions/examplesを前、variable contentを後へ置くことを推奨する。

candidate prompt layers:

~~~text
Stable prefix
  platform/system contract
  repository AGENTS
  role contract
  packet schema semantics
  tool contract

Task-bound middle
  Task Packet
  Context Pack manifest
  selected source excerpts

Variable suffix
  latest user input
  runtime state
  current diagnostics
~~~

Cache goals:

- stable prefixのhashを記録する。
- tool list/orderもcache identityへ含める。
- generated timestampやvolatile statusをprefixへ入れない。
- role別prefixを少数に保つ。
- cache hitをqualityより優先しない。

Q-CTX-002:

AGENTS、operating protocol、packet templateの重複をどう減らしつつ、agentへ必要なdurable instructionを渡すか。

candidate:

- AGENTS: repository invariant and commands
- protocol: role-independent workflow semantics
- packet: task-specific constraints
- runtime profile: model/sandbox/effort
- generated prompt: composition result

## 9. Long-Running Context And Compaction

documented: Responses API compactionはlong-running contextを縮小し、opaque compaction itemを次turnへ運ぶ。opaque itemはhuman-readable evidenceではない。

decision-candidate:

- compaction stateをaudit recordの正本にしない。
- durable task/decision/evidenceはexplicit packet/objectへ書き出す。
- resume前にbase、packet、open blockers、validation freshnessを再確認する。
- context windowが大きくてもtoken budgetを設定する。

Local observation:

- current bundled GPT-5.6 Sol catalogはこのCodex runtimeで372,000 context tokens、95 percent effective windowを報告する。
- provider/APIの公開context limitとlocal runtime effective limitは一致するとは限らない。

したがってruntime capability probeとpack budgetを別に持つ。

## 10. Freshness And Invalidation

Context Pack invalidation triggers:

- base tree digest changed
- required file/Atom digest changed
- policy/schema changed
- command/source map changed
- task scope changed
- dependency packet superseded
- new blocking issue added

Review Packet invalidation triggers:

- patch/head changed
- acceptance criteria changed
- required gate result expired
- evidence object missing/untrusted
- reviewer conflict requiring re-adjudication

Freshnessはtimestampだけでなくinput digestで判定する。

## 11. Security And Privacy

Context classification candidate:

| Class | Handling |
|---|---|
| public | external provider allowed per policy |
| repository | approved project runtime only |
| confidential | explicit provider/tool allowlist |
| secret | never place in model context; use reference/credential boundary |
| regulated | managed retention、region、audit policy |

Controls:

- secrets and auth filesをselectorで除外する。
- MCP resource/tool outputをuntrusted contentとして区切る。
- instruction-like repository textをpolicy instructionへ昇格しない。
- external URL fetchはdomain/approval policyを通す。
- prompt/tool traceのcontent captureはopt-in。
- context omissionにsecurity reasonを記録するがsecret valueは記録しない。

## 12. Metrics

- MET-AI-010: task完成までに読んだunique token/bytes
- MET-AI-011: context entry precision
- MET-AI-012: critical source recall
- MET-AI-013: stale context usage count
- MET-AI-014: prompt cache hit ratio
- MET-AI-015: redundant file read ratio
- MET-AI-016: time to first correct edit
- MET-AI-017: reviewer-requested missing context count
- MET-AI-018: packet clarification count
- MET-AI-019: scope violation count

precisionだけを高めてcritical recallを落とさない。

## 13. Decision Candidates And Promotion

- DC-AI-040: Context/Task/Review/Handoffをversioned typed contractへ移す。
- DC-AI-041: packet、context、patch、reviewをdigest bindする。
- DC-AI-042: authority、selection reason、omission、freshness、token budgetをContext Packへ追加する。
- DC-AI-043: Markdown packetをcanonical JSONから生成するか、round-trip lint可能にする。
- DC-AI-044: stable prompt prefixとvariable suffixを分けてcacheを測定する。
- DC-AI-045: opaque compactionをaudit evidenceとして扱わない。

Promotion先:

- docs/development/ai/operating-protocol.md
- docs/development/ai/context-packs.json replacement or v2
- schemas/codefire.*.json
- packet/context generators
- AI docs checker and conformance tests

## 14. Primary References

- [OpenAI Prompt Caching](https://developers.openai.com/api/docs/guides/prompt-caching)
- [OpenAI Compaction](https://developers.openai.com/api/docs/guides/compaction)
- [Codex AGENTS.md](https://developers.openai.com/codex/agent-configuration/agents-md)
- [JSON Schema specification](https://json-schema.org/specification)

## 15. Promotion Status

- Status: not promoted
- Required evidence: EXP-AI-002、EXP-AI-003、EXP-AI-013
- Revisit when: typed packet/context prototypeをhistorical task corpusで比較したとき

