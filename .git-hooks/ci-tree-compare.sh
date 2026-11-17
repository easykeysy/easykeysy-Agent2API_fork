#!/usr/bin/env bash
# ci-tree-compare.sh - LAW-11.R3 non-bypassable backstop.
#
# Local hooks (pre-commit Gate 6, pre-push/post-tag) can be skipped with
# --no-verify; this script re-checks the same facts server-side in CI, where
# no local flag can reach. It verifies, against the pinned upstream base:
#   1. manifest honesty  - .git-hooks/upstream-base.manifest is exactly the
#                          tree of the pinned upstream tag (regenerated live)
#   2. base drift        - the pinned tag still resolves to the pinned commit
#   3. pin lineage       - the base commit is an ancestor of the checked commit
#   4. allowlist valid   - every wiring-allowlist entry is a real base path
#   5. tree compare      - every upstream path in the checked tree matches its
#                          base blob unless the path is allowlisted; deleting
#                          an upstream path likewise requires the allowlist
#   6. live tags         - (CI only) the pinned tag still exists upstream
#
# Usage:
#   ci-tree-compare.sh                      # CI mode: fetches the pinned tag
#                                           # from UPSTREAM_URL, checks HEAD
#   ci-tree-compare.sh <upstream-ref> \     # offline mode: use a local ref as
#     [current-ls-tree-file]                # the base and (optionally) a file
#                                           # of `git ls-tree -r` output as the
#                                           # current tree (for testing)
set -uo pipefail

fail() { echo "ci-tree-compare: FAIL: $*" >&2; exit 1; }

MANIFEST=".git-hooks/upstream-base.manifest"
ALLOWLIST=".git-hooks/wiring-allowlist"
UPSTREAM_URL="https://github.com/aimod-cc/Agent2API.git"

[ -f "$MANIFEST" ] || fail "missing $MANIFEST (run from the repo root)"
[ -f "$ALLOWLIST" ] || fail "missing $ALLOWLIST (run from the repo root)"

UPREF="${1:-}"
CURFILE="${2:-}"
LIVE=0
[ -z "$UPREF" ] && LIVE=1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# ---- resolve the pinned base from the manifest header -----------------------
BASE_LINE="$(grep -m1 'pinned base: upstream' "$MANIFEST")" \
  || fail "manifest header lacks a 'pinned base: upstream <tag> = <sha>' line"
BASE_TAG="$(printf '%s\n' "$BASE_LINE" | sed -n 's/.*pinned base: upstream \([^ =]*\) = .*/\1/p')"
BASE_SHA="$(printf '%s\n' "$BASE_LINE" | sed -n 's/.*= \([0-9a-f]\{40\}\).*/\1/p')"
[ -n "$BASE_TAG" ] && [ -n "$BASE_SHA" ] \
  || fail "cannot parse base tag/commit from manifest header: $BASE_LINE"

# ---- resolve the upstream side ----------------------------------------------
if [ "$LIVE" = 1 ]; then
  git update-ref -d refs/ci-upstream-base 2>/dev/null || true
  git fetch --quiet --no-tags --filter=blob:none "$UPSTREAM_URL" \
    "refs/tags/$BASE_TAG:refs/ci-upstream-base" \
    || fail "could not fetch pinned base tag '$BASE_TAG' from $UPSTREAM_URL"
  UPREF=refs/ci-upstream-base
fi
UP_COMMIT="$(git rev-parse -q --verify "$UPREF^{commit}")" \
  || fail "upstream ref '$UPREF' does not resolve to a commit"

# ---- check 1: manifest honesty ----------------------------------------------
regen() { git ls-tree -r "$1" | sed -E 's/^[0-7]+ blob ([0-9a-f]{40})\t/\1\t/'; }
regen "$UPREF" | LC_ALL=C sort > "$TMP/regen.manifest" \
  || fail "cannot list the tree of $UPREF"
grep -v '^#' "$MANIFEST" | grep -v '^[[:space:]]*$' | LC_ALL=C sort \
  > "$TMP/committed.manifest"
if ! cmp -s "$TMP/regen.manifest" "$TMP/committed.manifest"; then
  fail "manifest honesty: .git-hooks/upstream-base.manifest does not match the tree of pinned base $BASE_TAG (via $UPREF). Regenerate the manifest at the base bump (LAW.md LAW-11.R3)."
fi
echo "ci-tree-compare: PASS manifest honesty ($(grep -vc '^#' "$TMP/committed.manifest") paths match upstream $BASE_TAG)"

# ---- check 2: base drift ------------------------------------------------------
[ "$UP_COMMIT" = "$BASE_SHA" ] \
  || fail "base drift: manifest pins $BASE_SHA but upstream tag '$BASE_TAG' now resolves to $UP_COMMIT"
echo "ci-tree-compare: PASS base drift ($BASE_TAG = $BASE_SHA)"

# ---- check 3: pin lineage (only when the current tree really is HEAD) --------
if [ -z "$CURFILE" ]; then
  git merge-base --is-ancestor "$UP_COMMIT" HEAD 2>/dev/null \
    || fail "pin lineage: base commit $BASE_SHA ($BASE_TAG) is not an ancestor of HEAD"
  echo "ci-tree-compare: PASS pin lineage ($BASE_TAG is an ancestor of HEAD)"
fi

# ---- check 4: allowlist validity ---------------------------------------------
while IFS= read -r p; do
  case "$p" in ''|\#*) continue ;; esac
  awk -F '\t' -v q="$p" '$1 ~ /^#/ {next} $2 == q {f=1} END {exit !f}' "$MANIFEST" \
    || fail "allowlist validity: '$p' is not a path in the base manifest (the allowlist may only shrink and may only contain real base paths)"
done < "$ALLOWLIST"
echo "ci-tree-compare: PASS allowlist validity ($(grep -vc '^#' "$ALLOWLIST") entries, all real base paths)"

# ---- check 5: tree compare ----------------------------------------------------
if [ -n "$CURFILE" ]; then
  cp "$CURFILE" "$TMP/current.manifest" || fail "cannot read current-tree file '$CURFILE'"
else
  regen HEAD > "$TMP/current.manifest" || fail "cannot list the tree of HEAD"
fi
awk -F '\t' '
  FNR==1 { fn++ }
  fn==1 && $0 !~ /^#/ && $0 != "" { base[$2]=$1; next }
  fn==2 && $0 !~ /^#/ && $0 != "" { allow[$0]=1; next }
  fn==3 { cur[$2]=$1; next }
  END {
    bad=0; shown=0
    for (p in base) {
      if (p in allow) continue
      if (!(p in cur)) { if (shown<20) print "  DELETED upstream path: " p; bad++; shown++; continue }
      if (cur[p] != base[p]) { if (shown<20) print "  MODIFIED upstream path: " p; bad++; shown++ }
    }
    if (bad>20) print "  ... and " (bad-20) " more"
    if (bad>0) { print "  " bad " upstream path(s) altered outside the wiring allowlist"; exit 1 }
    print "  all " length(base) " upstream paths unmodified or allowlisted"
    exit 0
  }
' "$TMP/committed.manifest" "$ALLOWLIST" "$TMP/current.manifest" \
  || fail "LAW-11 tree compare failed (list above). Fork work must go in standalone files; allowlist changes need maintainer sign-off."
echo "ci-tree-compare: PASS tree compare"

# ---- check 6: live tags (CI only) ---------------------------------------------
if [ "$LIVE" = 1 ]; then
  gh api "repos/aimod-cc/Agent2API/tags?per_page=100" --paginate \
    --jq '.[] | .commit.sha + "\t" + .name' > "$TMP/live.tags" 2>/dev/null \
    || fail "could not query the upstream tag list via gh api"
  awk -F '\t' -v t="$BASE_TAG" -v s="$BASE_SHA" \
    '$2==t {f=1; if ($1!=s) m=1} END {exit (f && !m) ? 0 : 1}' "$TMP/live.tags" \
    || fail "live tags: pinned base tag '$BASE_TAG' is missing upstream or re-pointed there"
  echo "ci-tree-compare: PASS live tags ($BASE_TAG still points at $BASE_SHA upstream)"
fi

echo "ci-tree-compare: ALL CHECKS PASSED (LAW-11 backstop, base $BASE_TAG)"
exit 0
