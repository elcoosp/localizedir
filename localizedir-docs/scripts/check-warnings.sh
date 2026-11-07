#!/bin/bash
# Zero-warning gate for pnpm build output (.qa/build.log).
#
# Every `[WARN]`/`warning` line is a release blocker, with exactly one
# allowlisted exception documented in docs/decisions/0005-known-warnings.md:
# the Astro route-priority warning emitted by Starlight's own custom-404
# recipe (src/content/docs/404.md). Full-string narrow match below; anything
# else fails.
set -euo pipefail
LOG="${1:-.qa/build.log}"

allowlisted=$(grep -F 'Could not render `/404` from route' "$LOG" | grep -c '\[WARN\]' || true)
total=$(grep -Eic '\b(warn|warning)\b' "$LOG" || true)
unexpected=$((total - allowlisted))

echo "warning lines: $total (allowlisted: $allowlisted)"
if [ "$unexpected" -gt 0 ]; then
	echo "UNEXPECTED WARNINGS — release blocker:"
	grep -Ein '\b(warn|warning)\b' "$LOG" | grep -vF 'Could not render `/404` from route'
	exit 1
fi
echo "OK: no unexpected warnings"
