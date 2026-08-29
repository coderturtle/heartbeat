#!/usr/bin/env bash
# check-brand-lint.sh: Detect brand/style violations (em dashes, banned phrases)
# in published workshop content. Read-only. This is a verification loop per
# modules/README.md's gate-tier vocabulary and gremlins/workshop/
# workshop-lifecycle.md's "Dogfooding" section: bounded scope, a fixed
# pass/fail check, human-readable output. Adapted directly from
# borrow-native's scripts/check-brand-lint.sh.
#
# Scope: published content only, per docs/brand.md's own boundary. Design/
# planning docs under docs/ are working documents and are explicitly exempt.
#
# Usage:
#   scripts/check-brand-lint.sh           # human report
#   scripts/check-brand-lint.sh --check   # hook mode: exit 1 if violations found
#
# Network: none. Pure local file inspection.
set -uo pipefail

CHECK_MODE=false
[[ "${1:-}" == "--check" ]] && CHECK_MODE=true

ROOT=""
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while [[ "$dir" != "/" ]]; do
  if [[ -f "$dir/.hekton/project.yaml" ]]; then ROOT="$dir"; break; fi
  dir="$(dirname "$dir")"
done
if [[ -z "$ROOT" ]]; then
  echo "check-brand-lint: no .hekton/project.yaml found above this script." >&2
  exit 0
fi
cd "$ROOT"

VIOLATIONS=0
warn() { printf '  FAIL: %s\n' "$*"; VIOLATIONS=$((VIOLATIONS + 1)); }
ok() { printf '  OK: %s\n' "$*"; }

# Published-content scope: README, module content, build-log entries, and the
# site's own source once built. Add new content dirs here as the workshop
# grows past its current skeleton.
SCOPE_FILES=()
[[ -f README.md ]] && SCOPE_FILES+=("README.md")
while IFS= read -r -d '' f; do SCOPE_FILES+=("$f"); done < <(find modules -name '*.md' -print0 2>/dev/null)
while IFS= read -r -d '' f; do SCOPE_FILES+=("$f"); done < <(find docs/build-log -name '*.md' -print0 2>/dev/null)
# Takeaway artifacts: learner-facing (dropped into their own harness), not internal scaffolding.
while IFS= read -r -d '' f; do SCOPE_FILES+=("$f"); done < <(find .claude/commands -name '*.md' -print0 2>/dev/null)
while IFS= read -r -d '' f; do SCOPE_FILES+=("$f"); done < <(find .claude/skills -name '*.md' -print0 2>/dev/null)
while IFS= read -r -d '' f; do SCOPE_FILES+=("$f"); done < <(find site/src -type f \( -name '*.astro' -o -name '*.mdx' \) -print0 2>/dev/null)

echo "-- Brand lint (published content only) ----------------------------------"
echo "  files checked: ${#SCOPE_FILES[@]}"
echo ""

if [[ "${#SCOPE_FILES[@]}" -eq 0 ]]; then
  ok "no published-content files found yet"
else
  # Hard rule: no em dash characters (docs/brand.md).
  EM_HITS=$(grep -lF '—' "${SCOPE_FILES[@]}" 2>/dev/null || true)
  if [[ -n "$EM_HITS" ]]; then
    warn "em dash found in: $(echo "$EM_HITS" | tr '\n' ' ')"
  else
    ok "no em dashes in published content"
  fi

  # Banned phrases (docs/brand.md's list, kept in sync by hand -- update both
  # when one changes).
  BANNED=(
    "delve" "tapestry" "unlock" "seamless" "game-changing" "revolutioniz"
    "transform your workflow" "supercharge" "effortlessly" "cutting-edge"
    "thought leader" "in today's fast-paced world" "it's important to note"
    "master the art of" "in this comprehensive guide" "10x your skills"
    "split-brain-proof" "bulletproof consensus"
  )
  for phrase in "${BANNED[@]}"; do
    HITS=$(grep -liF "$phrase" "${SCOPE_FILES[@]}" 2>/dev/null || true)
    if [[ -n "$HITS" ]]; then
      warn "banned phrase \"$phrase\" found in: $(echo "$HITS" | tr '\n' ' ')"
    fi
  done
  [[ "$VIOLATIONS" -eq 0 ]] && ok "no banned phrases in published content"
fi

echo ""
if [[ "$VIOLATIONS" -eq 0 ]]; then
  echo "Brand lint clean."
else
  echo "Brand lint found $VIOLATIONS issue(s). Fix per docs/brand.md's hard rules."
fi

if [[ "$CHECK_MODE" == true ]]; then
  [[ "$VIOLATIONS" -gt 0 ]] && exit 1
fi
exit 0
