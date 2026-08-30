#!/usr/bin/env bash
# Checks pending (uncommitted) changes against .hekton/governance.yaml's
# arb_review_triggers list. Run before starting work on fixtures/checkout/'s
# already-shipped shared files (Coachgremlin Workflow step 0,
# <hekton-machinery>/gremlins/coaching/coachgremlin.md) - a later module's feature can
# silently change or break an earlier module's already-graded gate.
set -euo pipefail

DRY_RUN=0
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GOVERNANCE="$ROOT_DIR/.hekton/governance.yaml"
LOG_DIR="$ROOT_DIR/.project-setup/logs"
LOG_FILE="$LOG_DIR/arb-trigger-check.log"

usage() {
  echo "Usage: $0 [--dry-run]"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
  esac
done

log() {
  echo "$1"
  if [ "$DRY_RUN" -eq 0 ]; then
    mkdir -p "$LOG_DIR"
    printf "%s %s\n" "$(date '+%Y-%m-%d %H:%M:%S %Z')" "$1" >> "$LOG_FILE"
  fi
}

if [ ! -f "$GOVERNANCE" ]; then
  log "MISSING: $GOVERNANCE - nothing to check against"
  exit 1
fi

# Pull the arb_review_triggers list out of governance.yaml. Each entry is a
# `pattern:`/`reason:`/`review_type:` triple, in that order - parsed
# positionally rather than with a YAML library to keep this script
# dependency-free.
#
# Known limitation (inherited from borrow-native's own version of this
# script, flagged 2026-07-05, Modules 03+04 Workshop Review Panel batch,
# Security-Conscious Reviewer persona): the regexes below only match
# double-quoted values in this exact key order. A governance.yaml reformatted
# with single quotes, folded/multiline values, or reordered keys would fail
# to match silently - patterns/reasons/review_types would stay empty and this
# script would report "no triggers configured" (exit 0) rather than erroring
# loudly. Verified against the current governance.yaml's actual shape (still
# matches); not yet hardened against format drift - a real YAML parser would
# close this, at the cost of the dependency-free property above.
patterns=()
reasons=()
review_types=()
current_pattern=""
current_reason=""
in_triggers=0
while IFS= read -r line; do
  if [[ "$line" == "arb_review_triggers:"* ]]; then
    in_triggers=1
    continue
  fi
  if [ "$in_triggers" -eq 1 ]; then
    if [[ "$line" =~ ^[A-Za-z_]+: ]]; then
      break # left the arb_review_triggers block
    fi
    if [[ "$line" =~ pattern:\ \"(.*)\" ]]; then
      current_pattern="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ reason:\ \"(.*)\" ]]; then
      current_reason="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ review_type:\ (.*) ]]; then
      patterns+=("$current_pattern")
      reasons+=("$current_reason")
      review_types+=("${BASH_REMATCH[1]}")
    fi
  fi
done < "$GOVERNANCE"

if [ "${#patterns[@]}" -eq 0 ]; then
  log "No arb_review_triggers configured in $GOVERNANCE"
  exit 0
fi

# Each porcelain line's path field can be a plain path, a quoted path (if it
# contains a space), or `old -> new` for a rename - check all path-shaped
# tokens on the line, not just the naive second field, so a rename into a
# protected path or a quoted path with a space isn't silently missed.
changed_files="$(cd "$ROOT_DIR" && git status --porcelain \
  | sed -E 's/^.{3}//' \
  | sed -E 's/^"(.*)"$/\1/' \
  | sed -E 's/.* -> //' )"

fired=0
for i in "${!patterns[@]}"; do
  pattern="${patterns[$i]}"
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    if [ "$f" = "$pattern" ]; then
      log "ARB TRIGGER FIRED: $f matches '$pattern' (review_type: ${review_types[$i]}) - ${reasons[$i]}"
      fired=1
    fi
  done <<< "$changed_files"
done

if [ "$fired" -eq 0 ]; then
  log "No ARB triggers fired against current pending changes."
  exit 0
fi

# Non-zero on a fire: this check exists so a trigger blocks (a hook, a CI
# step, or Coachgremlin's own Workflow step 0) rather than just logging a
# warning nobody is forced to read.
exit 1
