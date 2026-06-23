#!/bin/bash
# PID 1 for the Complement perf testee. Runs the server under `perf stat` but
# forwards the container stop signal to the server (perf's child) so the server
# shuts down gracefully: its runtime-metrics dump fires on Drop, and perf then
# flushes its counter report once the workload exits.
#
# Wrapping the server directly under perf instead loses both, because perf as
# PID 1 does not propagate the stop signal to its workload: the server is killed
# ungracefully on the grace timeout and perf writes an empty report.
#
# The counter report lands in the runtime-metrics directory so the per-test
# extraction in post_test.sh collects it alongside the JSON dumps.
set -eo pipefail

out="${TUWUNEL_PERF_STAT_OUT:-/var/log/tuwunel/metrics/tuwunel.perf_stat.csv}"

perf stat -ddd -x, --output "$out" -- "$@" &
perf_pid=$!

forward() {
	local kids
	kids=$(cat "/proc/$perf_pid/task/$perf_pid/children" 2>/dev/null) || kids=""

	for kid in $kids; do
		kill -TERM "$kid" 2>/dev/null || true
	done
}
trap forward TERM INT

# wait returns early when a trapped signal arrives; loop until perf truly exits
# so its counter report is fully flushed before this PID exits.
ec=0
wait "$perf_pid" 2>/dev/null || ec=$?
while kill -0 "$perf_pid" 2>/dev/null; do
	wait "$perf_pid" 2>/dev/null || ec=$?
done

exit "$ec"
