#!/usr/bin/env bash
set -euo pipefail

CODEFIRE=${CODEFIRE:-"$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/codefire"}
workdir=${1:-"$(mktemp -d)"}

write_demo_files() {
  local dir=$1
  local minutes=$2
  mkdir -p "$dir/docs/spec" "$dir/docs/design" "$dir/src" "$dir/tests"
  cat > "$dir/docs/spec/auth.md" <<EOF
## REQ-AUTH-001: Session expiration

Sessions expire after $minutes minutes.
EOF
  cat > "$dir/docs/design/auth.md" <<EOF
## DES-AUTH-001: Session policy

SessionPolicy returns $minutes minutes.
EOF
  cat > "$dir/src/auth.py" <<EOF
# cf-atom: CODE-SessionPolicy
class SessionPolicy:
    def expires_after_minutes(self):
        return $minutes
EOF
  cat > "$dir/tests/test_auth.py" <<EOF
from src.auth import SessionPolicy

# cf-atom: TEST-session-expiration
def test_session_expiration():
    assert SessionPolicy().expires_after_minutes() == $minutes
EOF
  cat > "$dir/codefire.yaml" <<'EOF'
version: 1
artifacts:
  requirements:
    - path: docs/spec/**/*.md
      kind: requirement_document
  designs:
    - path: docs/design/**/*.md
      kind: design_document
  code:
    - path: src/**/*.py
      kind: python_code
  tests:
    - path: tests/**/*.py
      kind: pytest_test
EOF
  cat > "$dir/codefire.links.yaml" <<'EOF'
version: 1
links:
  - from: REQ-AUTH-001
    to: DES-AUTH-001
    type: refined_by
  - from: DES-AUTH-001
    to: CODE-SessionPolicy
    type: implemented_by
  - from: REQ-AUTH-001
    to: TEST-session-expiration
    type: verified_by
EOF
  cat > "$dir/codefire.policy.yaml" <<'EOF'
version: 1
verification:
  required:
    - id: unit-tests
      command: "python3 -m unittest discover -s tests"
      cwd: "."
EOF
}

extinguish_all() {
  local dir=$1
  local evidence=$2
  while read -r fire_id; do
    [ -n "$fire_id" ] || continue
    "$CODEFIRE" extinguish "$fire_id" --resolution changed --evidence "$evidence"
  done < <("$CODEFIRE" scan | awk '/^  FIRE-/ {print $1}')
}

echo "workdir: $workdir"
mkdir -p "$workdir"
cd "$workdir"

"$CODEFIRE" init
"$CODEFIRE" open main "$workdir/main"
cd "$workdir/main"
write_demo_files "$PWD" 30
extinguish_all "$PWD" "initial consistent import"
"$CODEFIRE" verify
"$CODEFIRE" commit -m "Initial consistent auth sample"

cd "$workdir"
"$CODEFIRE" clone main feature-session
"$CODEFIRE" open feature-session "$workdir/feature-session"
cd "$workdir/feature-session"
write_demo_files "$PWD" 15
extinguish_all "$PWD" "change session expiration to 15 minutes"
"$CODEFIRE" verify
"$CODEFIRE" commit -m "Change session expiration to 15 minutes"

cd "$workdir"
"$CODEFIRE" merge feature-session --into main
cd "$workdir/main"
extinguish_all "$PWD" "merge feature-session"
"$CODEFIRE" verify
"$CODEFIRE" commit -m "Merge feature-session"

server="$workdir/server"
project_url="cf://$server/org/app"
cd "$workdir"
"$CODEFIRE" upload main "$project_url/main"
"$CODEFIRE" upload feature-session "$project_url/feature-session"
mr_output=$("$CODEFIRE" request-merge "$project_url/main" "$project_url/feature-session")
echo "$mr_output"
mr_id=$(printf '%s\n' "$mr_output" | awk '/^created MR-/ {print $2}')
"$CODEFIRE" request-review "$project_url" "$mr_id" --reviewer demo --decision approve
"$CODEFIRE" request-apply "$project_url" "$mr_id"
"$CODEFIRE" doctor "$project_url"

echo "CodeFire demo completed."
