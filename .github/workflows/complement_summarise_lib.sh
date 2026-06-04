#!/bin/bash
# Shared helpers for the per-flavour complement summariser drivers. Sourced;
# not invoked directly. Drivers set:
#
#   track_name      Heading text (e.g. "Complement", "Complement-Crypto").
#   jsonl           Path to the per-flavour results.jsonl.
#   metrics_tar     Optional. Path to per-run runtime_metrics.tar.zst. When
#                   set, emit_runtime_metrics() will be wired into main.
#   artifact_name   Optional. Name of the per-flavour runtime-metrics
#                   artifact on the prior successful main.yml run. Required
#                   when metrics_tar is set.
#
# Drivers then call `summarise_main "$@"`.

set -eo pipefail

out="${GITHUB_STEP_SUMMARY:-/dev/stdout}"

# Some tests emit both pass and fail rows (subtests). Fold by Test, fail wins.
classify() {
	jq -sr '
		group_by(.Test)[]
		| { t: .[0].Test, s:
		      (if   any(.[]; .Action == "fail") then "error"
		       elif any(.[]; .Action == "pass") then "accept"
		       else                                  "skip"
		       end) }
		| "\(.s) \(.t)"
	' "$1"
}

snapshot_current() {
	classify "$jsonl" | sort -k2 > "$curr";
}

snapshot_baseline() {
	git show "HEAD:$jsonl" 2>/dev/null | classify /dev/stdin | sort -k2 > "$prev" || :;
}

# Count test names in a newline-joined list, split by section (no '/') vs
# subtest (the "X/Y" rows Go emits beneath a top-level test).
count_sec() {
	printf '%s' "$1" | grep -c '^[^/]*$' || :;
}

count_sub() {
	printf '%s' "$1" | grep -c '/' || :;
}

delta() {
	join -j2 -t ' ' \
		<(awk -v k="$1" '$1==k' "$prev") \
		<(awk -v k="$2" '$1==k' "$curr") \
		| cut -d' ' -f1
}

# Fixed column count for every flavour's grid, anchored to the Complement
# track's square width (ceil(sqrt(791)) = 29 at time of writing). Holding the
# width constant lets the smaller Complement-Crypto suite render as a short
# horizontal band instead of a tiny square; grid height then floats with each
# flavour's test count. A driver may override by setting grid_width first.
grid_width="${grid_width:-29}"

# Cells: ❎/🟩 for accept (progressed vs stable), 🟥/🟧 for error (regressed vs known),
# ⬜ skip. With no baseline (nobase=1) all fails are red and all passes plain green.
render_grid() {
	awk -v w="$1" -v nobase="$2" -v R="$3" -v P="$4" '
		BEGIN {
			split(R, x, "\n"); for (i in x) if (x[i]) reg[x[i]] = 1
			split(P, x, "\n"); for (i in x) if (x[i]) pro[x[i]] = 1
		}
		{
			t = $2
			if      ($1 == "skip")   c = "⬜"
			else if ($1 == "accept") c = pro[t]             ? "❎" : "🟩"
			else if ($1 == "error")  c = (nobase || reg[t]) ? "🟥" : "🟧"
			printf "%s", c
			if (++n % w == 0) printf "<br>"
		}
		END {
			while (n % w != 0) { printf "⬛"; if (++n % w == 0) printf "<br>" }
		}
	' "$curr"
}

emit_header() {
	echo "### $track_name"
	echo
	echo "|  | accept | errors | skipped | advanced | regressed |"
	echo "|---|---|---|---|---|---|"
	echo "| sections | $acc_sec | $err_sec | $skip_sec | $nprog_sec | $nreg_sec |"
	echo "| subtests | $acc_sub | $err_sub | $skip_sub | $nprog_sub | $nreg_sub |"
	if test -n "$1"; then
		echo
		echo "$1"
	fi
}

emit_diff() {
	test -n "$regress$progress" || return 0
	echo
	echo '```diff'
	printf '%s\n' "$regress"  | sed -n 's/^./- &/p'
	printf '%s\n' "$progress" | sed -n 's/^./+ &/p'
	echo '```'
}

# Fetch the prior successful main.yml run's runtime-metrics artifact for this
# matrix slot. Echoes the local tar path and the run id on success, nothing
# on miss. Soft-fails: no gh, no auth, no prior run, expired artifact all
# yield "".
fetch_baseline_metrics() {
	command -v gh >/dev/null 2>&1 || return 0
	test -n "${feat_set:-}${sys_name:-}${sys_target:-}" || return 0
	test -n "${artifact_name:-}" || return 0

	local branch="${GITHUB_REF_NAME:-}"
	test "$branch" = "main" && return 0   # no useful comparand on main itself

	local artifact="${artifact_name}-${feat_set}-${sys_name}-${sys_target}.tar.zst"
	local prev
	prev=$(gh run list \
		--workflow=main.yml \
		--branch="$branch" \
		--status=success \
		--limit=5 \
		--json databaseId \
		--jq '.[].databaseId' 2>/dev/null \
		| grep -v "^${GITHUB_RUN_ID:-}\$" \
		| head -1) || return 0
	test -n "$prev" || return 0

	local dir
	dir=$(mktemp -d)
	if ! gh run download "$prev" --name "$artifact" --dir "$dir" >/dev/null 2>&1; then
		rm -rf "$dir"
		return 0
	fi
	test -s "$dir/runtime_metrics.tar.zst" || { rm -rf "$dir"; return 0; }
	printf '%s\t%s\n' "$dir/runtime_metrics.tar.zst" "$prev"
}

emit_runtime_metrics() {
	test -n "${metrics_tar:-}" || return 0
	test -s "$metrics_tar" || return 0
	command -v python3 >/dev/null 2>&1 || return 0

	local script="$(dirname "$BASH_SOURCE")/complement_metrics_summarise.py"
	test -x "$script" || return 0

	local pair base_tar base_run
	pair=$(fetch_baseline_metrics || true)
	base_tar=$(printf '%s' "$pair" | cut -f1)
	base_run=$(printf '%s' "$pair" | cut -f2)

	local args=(--tar "$metrics_tar" --out "$out")
	if test -n "$base_tar" && test -s "$base_tar"; then
		args+=(--baseline-tar "$base_tar" --baseline-label "Baseline (run $base_run)")
	fi
	python3 "$script" "${args[@]}" || :
	if test -n "$base_tar"; then
		rm -rf "$(dirname "$base_tar")"
	fi
}

summarise_main() {
	if test ! -s "$jsonl"; then
		echo "No results.jsonl produced." >> "$out"
		exit 0
	fi

	curr=$(mktemp); prev=$(mktemp)
	trap 'rm -f "$curr" "$prev"' EXIT
	snapshot_current
	snapshot_baseline

	regress=$( delta accept error)
	progress=$(delta error  accept)

	nobase=0
	test -s "$prev" || { regress= progress= nobase=1; }

	# Tallies split by section (top-level test) vs subtest. The accept/error/
	# skip counts come from the classified snapshot; advanced/regressed are the
	# diff lists partitioned by the same '/' discriminator.
	read -r acc_sec err_sec skip_sec acc_sub err_sub skip_sub < <(awk '
		{ is_sub = ($2 ~ /\//) }
		$1 == "accept" { if (is_sub) sub_a++; else sec_a++ }
		$1 == "error"  { if (is_sub) sub_e++; else sec_e++ }
		$1 == "skip"   { if (is_sub) sub_k++; else sec_k++ }
		END { printf "%d %d %d %d %d %d\n",
		             sec_a + 0, sec_e + 0, sec_k + 0,
		             sub_a + 0, sub_e + 0, sub_k + 0 }
	' "$curr")

	nprog_sec=$(count_sec "$progress"); nprog_sub=$(count_sub "$progress")
	nreg_sec=$( count_sec "$regress");  nreg_sub=$( count_sub "$regress")

	# Main-branch runs are the baseline; the grid carries no diff signal there.
	if test "${GITHUB_REF_NAME:-}" = "main"; then
		grid=
	else
		grid=$(render_grid "$grid_width" "$nobase" "$regress" "$progress")
	fi

	{ emit_header "$grid"; emit_diff; } >> "$out"
	emit_runtime_metrics
}
