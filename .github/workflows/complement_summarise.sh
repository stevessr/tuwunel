#!/bin/bash
set -eo pipefail

jsonl="tests/complement/results.jsonl"
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

count_status() {
	grep -c "^$1 " "$curr" || :;
}

count_lines() {
	printf '%s' "$1" | grep -c . || :;
}

delta() {
	join -j2 -t ' ' \
		<(awk -v k="$1" '$1==k' "$prev") \
		<(awk -v k="$2" '$1==k' "$curr") \
		| cut -d' ' -f1
}

grid_width() {
	awk -v n="$1" 'BEGIN { w=int(sqrt(n)); if (w*w<n) w++; print (w?w:1) }';
}

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
			while (n < w*w) { printf "⬛"; if (++n % w == 0) printf "<br>" }
		}
	' "$curr"
}

emit_header() {
	echo "### Complement"
	echo
	echo "| accept | errors | skipped | advanced | regressed |"
	echo "|---|---|---|---|---|"
	echo "| $accept | $error | $skip | $nprog | $nreg |"
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

main() {
	if test ! -s "$jsonl"; then
		echo "No results.jsonl produced." >> "$out"
		exit 0
	fi

	curr=$(mktemp); prev=$(mktemp)
	trap 'rm -f "$curr" "$prev"' EXIT
	snapshot_current
	snapshot_baseline

	accept=$(count_status accept)
	error=$( count_status error)
	skip=$(  count_status skip)
	total=$((accept + error + skip))

	regress=$( delta accept error)
	progress=$(delta error  accept)
	nreg=$( count_lines "$regress")
	nprog=$(count_lines "$progress")

	nobase=0
	test -s "$prev" || { regress= progress= nobase=1; }

	# Main-branch runs are the baseline; the grid carries no diff signal there.
	if test "${GITHUB_REF_NAME:-}" = "main"; then
		grid=
	else
		grid=$(render_grid "$(grid_width "$total")" "$nobase" "$regress" "$progress")
	fi

	{ emit_header "$grid"; emit_diff; } >> "$out"
}

main "$@"
