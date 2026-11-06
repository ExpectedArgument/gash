#!/usr/bin/env bash
# Realistic: option-parsing CLI with getopts, usage(), validation, and dispatch
PROG=mytool
VERSION=1.4.2

usage() {
  cat <<EOF
Usage: $PROG [-h] [-V] [-v] [-n COUNT] [-o FILE] [-m MODE] ARGS...
  -h        show help
  -V        show version
  -v        verbose (repeatable)
  -n COUNT  number of iterations (default 1)
  -o FILE   output file
  -m MODE   one of: fast, safe
EOF
}

die() { echo "error: $*" >&2; return 2; }

run() {
  local OPTIND opt verbose=0 count=1 outfile="" mode=safe
  while getopts ":hVvn:o:m:" opt; do
    case $opt in
      h) usage; return 0;;
      V) echo "  $PROG version $VERSION"; return 0;;
      v) (( verbose++ ));;
      n) if [[ $OPTARG =~ ^[0-9]+$ ]]; then count=$OPTARG; else die "bad count: $OPTARG"; echo "  -> bad count '$OPTARG'"; return 2; fi;;
      o) outfile=$OPTARG;;
      m) case $OPTARG in
           fast|safe) mode=$OPTARG;;
           *) echo "  -> invalid mode '$OPTARG'"; return 2;;
         esac;;
      :) echo "  -> option -$OPTARG requires an argument"; return 2;;
      \?) echo "  -> unknown option -$OPTARG"; usage | head -n 1; return 2;;
    esac
  done
  shift $((OPTIND - 1))
  if (( $# == 0 )); then echo "  -> no args"; return 1; fi
  echo "  verbose=$verbose count=$count outfile=${outfile:-<stdout>} mode=$mode"
  local i arg
  for (( i = 1; i <= count; i++ )); do
    for arg in "$@"; do
      (( verbose > 1 )) && echo "  [debug] iteration $i arg '$arg'"
      echo "  $i: ${arg^^}"
    done
  done
  if [[ $mode == fast && $count -gt 1 ]]; then
    echo "  (fast mode skipped $(( count - 1 )) redundant passes)"
  fi
  return 0
}

try() {
  echo "\$ $PROG $*"
  run "$@" 2>/dev/null
  echo "  (exit $?)"
}

try -h
try -V
try a b
try -v -n 2 x
try -vv -n2 -m fast "two words"
try -n abc x
try -m turbo x
try -o
try -z x
try -v
try -o out.txt -- -notanoption
try -vvv -n 1 last
echo "done: $PROG $VERSION"
