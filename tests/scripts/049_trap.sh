# trap: EXIT (main and subshell), ERR, RETURN, signals; script exits with 3
trap 'echo "c99: main EXIT trap, status $?"' EXIT
( trap 'echo "c1: subshell exit trap"' EXIT; echo "c1: in subshell" )
( trap 'echo "c2: trap on exit 3, status=$?"' EXIT; exit 3 ); echo "c2: subshell status $?"
trap -p EXIT | sed 's/^/c3: /'
f() { trap 'echo "c4: RETURN trap"' RETURN; echo "c4: in f"; }
f
trap - RETURN
( trap 'echo "c5: ERR at line $LINENO"' ERR; false; true; echo "c5: continues" )
( trap 'echo "c6: ERR"' ERR; if false; then :; fi; false || true; ! false; while false; do :; done; echo "c6: no ERR fired" )
( trap 'echo "c7: ERR status=$?"' ERR; (exit 4); echo "c7: after" )
( trap 'echo "c8: ERR in func"' ERR; g() { false; echo "c8: g continues"; }; g; echo "c8: after g" )
( set -E; trap 'echo "c9: ERR inherited ${FUNCNAME[0]-main}"' ERR; h() { false; }; h; echo "c9: done" )
( trap 'echo "c10: ERR"' ERR; set -e; false; echo never ); echo "c10: status $?"
( trap 'echo "c11: USR1 caught"' USR1; kill -USR1 $BASHPID; echo "c11: after kill" )
( trap '' INT; trap -p INT | sed 's/^/c12: /' )
( trap 'echo x' TERM; trap - TERM; echo "c13: reset [$(trap -p TERM)]" )
( trap 'echo "c14: first"' EXIT; trap 'echo "c14: replaced"' EXIT )
( trap 'echo "c15: exit in trap"; exit 9' EXIT; exit 1 ); echo "c15: status $?"
x=$( trap 'echo "c16: cmdsub trap"' EXIT; echo body ); echo "c16: [$x]"
( trap 'echo "c17: multi"' EXIT INT TERM; trap -p | sort | sed 's/^/c17: /' )
( cleanup() { echo "c18: cleanup function"; }; trap cleanup EXIT; echo "c18: work" )
( trap 'echo "c19: EXIT after set -e failure"' EXIT; set -e; false; echo never ); echo "c19: status $?"
( trap 'echo "c20: trap sees var [$v]"' EXIT; v=late )
( trap 'echo "c21: status in trap $?"' EXIT; false )
echo "c21: subshell exit status $?"
( trap "echo 'c22: double-quoted trap expanded early: $BASH_SUBSHELL'" EXIT )
( trap 'rc=$?; echo "c23: cleanup saw $rc"; exit $rc' EXIT; (exit 6) ; exit ); echo "c23: $?"
( trap 'echo "c24: ERR once"' ERR; false | true; echo "c24: pipeline ok" )
( trap 'echo "c25: ERR $?"' ERR; f25() { return 2; }; f25; echo "c25: done" )
( trap 'echo "c26: in EXIT"' EXIT; trap - EXIT; echo "c26: trap removed" )
echo "c27: last line before main exit"
exit 3
