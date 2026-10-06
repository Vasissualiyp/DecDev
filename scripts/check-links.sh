#!/usr/bin/env bash
# Link-check for every implementations[].url / demo_url in /components.
#
# Why this exists: an AI coding agent once filled the entire seed set with
# plausible-looking but entirely fabricated github.com/... URLs. "Don't
# invent links" as a written rule alone did not prevent that — this is the
# mechanical check, run in CI, per specs/05-validation-ci.md. A network
# check, deliberately NOT part of `cargo test` (flaky/slow/not what that
# suite is for) and NOT routed through decdev-core (this is link liveness,
# not schema validation).
#
# Requires: decdev (built/installed and on PATH), jq, curl.
set -euo pipefail

urls=$(decdev export | jq -r '.[].implementations[] | .url, (.demo_url // empty)' | sort -u)

failed=0

check_one() {
  local url="$1"
  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" -L --max-time 15 "$url" 2>/dev/null || echo "000")
  if [ "$code" -ge 200 ] && [ "$code" -lt 400 ]; then
    return 0
  fi
  return 1
}

while IFS= read -r url; do
  [ -z "$url" ] && continue
  if check_one "$url"; then
    continue
  fi
  # One retry before declaring it dead — transient network hiccups happen.
  sleep 2
  if check_one "$url"; then
    continue
  fi
  echo "DEAD LINK: $url" >&2
  failed=1
done <<<"$urls"

if [ "$failed" -ne 0 ]; then
  echo "One or more implementation links did not resolve. See specs/01-capability-spec-format.md — every url/demo_url must be real and verified, never invented." >&2
  exit 1
fi

echo "All implementation links resolve."
