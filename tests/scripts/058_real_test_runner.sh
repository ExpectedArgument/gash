#!/usr/bin/env bash
# Realistic: mini test runner with assertion helpers, discovery, subshell isolation and an EXIT trap.
# One test fails on purpose, so the script exits 1.
passed=0 failed=0
declare -a failures=()
summary() {
  echo "== summary: $passed passed, $failed failed =="
  if (( ${#failures[@]} > 0 )); then printf '   failed: %s\n' "${failures[@]}"; fi
}
trap summary EXIT

assert_eq() {
  local expected=$1 actual=$2 msg=${3:-}
  if [[ $expected == "$actual" ]]; then return 0; fi
  echo "    assertion failed: ${msg:+$msg: }expected [$expected] got [$actual]"
  return 1
}
assert_status() { local want=$1; shift; "$@"; local got=$?; assert_eq "$want" "$got" "status of $*"; }
assert_match() { [[ $2 =~ $1 ]] || { echo "    no match: /$1/ vs [$2]"; return 1; }; }

test_arith() { assert_eq 4 $((2 + 2)) && assert_eq 9 $((3 ** 2)); }
test_strings() { local s="Hello"; assert_eq HELLO "${s^^}" && assert_eq 5 "${#s}"; }
test_arrays() { local a=(1 2 3); assert_eq 3 "${#a[@]}" && assert_eq "2 3" "${a[*]:1}"; }
test_failing_on_purpose() { assert_eq apples oranges "fruit check"; }
test_status() { assert_status 1 false && assert_status 0 true; }
test_errexit_in_subshell() { ( set -e; false; echo unreachable ); assert_eq 1 $? "subshell errexit"; }
test_trap_in_cmdsub() { local out; out=$( trap 'echo cleaned' EXIT; echo work ); assert_eq $'work\ncleaned' "$out"; }
test_regex() { assert_match '^v[0-9]+\.[0-9]+$' "v1.22"; }
test_isolation() { GLOBAL_LEAK=1; assert_eq 1 "$GLOBAL_LEAK"; }
test_pipefail_set() { [[ -o pipefail ]]; }
not_a_test() { echo "should not run"; }

run_test() {
  local t=$1 rc
  ( set -o pipefail; "$t" )
  rc=$?
  if (( rc == 0 )); then
    printf '%-28s PASS\n' "$t"
    (( passed++ ))
  else
    printf '%-28s FAIL (rc=%d)\n' "$t" "$rc"
    (( failed++ ))
    failures+=("$t")
  fi
}

mapfile -t tests < <(declare -F | while read -r _ _ name; do [[ $name == test_* ]] && echo "$name"; done | sort)
echo "discovered ${#tests[@]} tests"
for t in "${tests[@]}"; do run_test "$t"; done
echo "GLOBAL_LEAK after run: [${GLOBAL_LEAK-unset}]"
exit $(( failed > 0 ? 1 : 0 ))
