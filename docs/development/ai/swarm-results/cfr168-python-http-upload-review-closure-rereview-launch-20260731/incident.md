# CFR-168 Python HTTP Upload Rereview Launch Incident

Recorded: 2026-07-31T03:46:00Z

Disposition: pre-provider launch failure; safe to retry with a new batch and
task identity.

## Failed attempt

| Field | Exact value |
|---|---|
| Batch | `cfr168-python-http-upload-review-closure-rereview-fugu-20260731` |
| Task | `python-http-upload-review-closure-rereview` |
| Manager control head | `3a935511246cd0fe57de526c8c85aa58f1588b90` |
| Task launch revision / tree | `11f5fc7fbeb1047d1a94096623e709350abc4a5d` / `1a144a88776d2e679395aa89e06445eafdfe86ae` |
| Candidate under review | `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Terminal scheduler result | `failed`, return code `3`, zero task commits |
| Task log SHA-256 | `2750571cbeee668dcf712f8c862c03747811913fa9fceb6f40e0cb042200b373` |
| State SHA-256 | `e3e8773339a0266ed3ac9cf6afeead34d1e79c4e304dbaf5a261e8c863de95df` |

The task stopped at the local readiness check with:

```text
error: Fugu is not ready; configure SAKANA_API_KEY/fugu-api or explicitly choose another worker.
```

No Fugu provider process or provider session was created. The generated
review worktree remained clean at the exact launch revision, the declared
report path was not created, and the failed attempt consumed no review
decision.

## Cause and retry control

The credential file exists only at `/workspace/project/fugu-api`, with mode
`0600`. The manager control worktree intentionally does not contain a copied
credential, and the launch environment did not name the external file through
`FUGU_API_KEY_FILE`. The readiness check therefore failed before provider
dispatch even though another already-running Fugu lane had inherited a
credential through its own earlier launch environment.

The retry must:

1. preserve the credential outside every Git worktree;
2. export only `FUGU_API_KEY_FILE=/workspace/project/fugu-api` to the new
   scheduler process, without recording or printing the file contents;
3. use a new batch ID, task ID, agent worktree, and Fugu session;
4. keep the same immutable candidate, ticket, and sole report write path;
5. repeat resource/provider/conflict admission immediately before launch; and
6. prove the exact `fugu-ultra-v1.1` provider process and durable task state
   after launch.

The failed state and logs remain preserved under:

`/workspace/worktrees/cfr168-python-http-upload-review-closure-rereview-fugu-base/.dev-swarm/cfr168-python-http-upload-review-closure-rereview-fugu-20260731/`

