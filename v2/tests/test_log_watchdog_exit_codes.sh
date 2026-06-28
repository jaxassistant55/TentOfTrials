#!/bin/sh
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
WATCHDOG="$ROOT/v2/scripts/log_watchdog.pl"
FIXTURES="$ROOT/v2/tests/fixtures/log_watchdog"

run_watchdog() {
  perl "$WATCHDOG" --scan-once "$@"
}

clean_output=$(run_watchdog "$FIXTURES/clean.log")
printf '%s' "$clean_output" | grep -q 'Alerts sent: 0'

warning_output=$(run_watchdog "$FIXTURES/warning.log")
printf '%s' "$warning_output" | grep -q 'TIMEOUT'
printf '%s' "$warning_output" | grep -q 'RATE_LIMIT'

set +e
error_output=$(run_watchdog "$FIXTURES/error.log")
error_status=$?
set -e
test "$error_status" -ne 0
printf '%s' "$error_output" | grep -q 'NULL_POINTER'
printf '%s' "$error_output" | grep -q 'FATAL_ERROR'

no_fail_output=$(perl "$WATCHDOG" --scan-once --no-fail "$FIXTURES/error.log")
printf '%s' "$no_fail_output" | grep -q 'NULL_POINTER'
printf '%s' "$no_fail_output" | grep -q 'FATAL_ERROR'
