# set -e behavior and its exceptions; script ends by failing under set -e
( set -e; echo "c1: start"; false; echo "c1: not reached" ); echo "c1: status $?"
( set -e; if false; then :; else echo "c2: if condition exempt"; fi; echo "c2: continues" )
( set -e; false && true; echo "c3: && list exempt" )
( set -e; false || echo "c4: || rescue" ; echo "c4: continues" )
( set -e; ! true; echo "c5: negation exempt" )
( set -e; f() { false; echo "c6: f continues in condition context"; }; if f; then echo "c6: f true"; fi )
( set -e; f() { false; echo "c7: not reached"; }; f; echo "c7: not reached either" ); echo "c7: status $?"
( set -e; f() { false; echo "c8: continues"; }; f || echo "c8: or"; )
( set -e; false | true; echo "c9: pipeline last cmd decides" )
( set -e; set -o pipefail; false | true; echo "c10: not reached" ); echo "c10: status $?"
( set -e; x=$(false); echo "c11: not reached" ); echo "c11: status $?"
( set -e; local_f() { local y=$(false); echo "c12: local masks"; }; local_f )
( set -e; x=$(false; echo "c13: cmdsub continues without inherit_errexit"); echo "$x" )
( set -e; shopt -s inherit_errexit; x=$(false; echo "c14: never"); echo "c14: after [$x]" ); echo "c14: status $?"
( set -e; while false; do :; done; echo "c15: while condition exempt" )
( set -e; until true; do :; done; echo "c16: until ok" )
( set -e; (false); echo "c17: not reached" ); echo "c17: status $?"
( set -e; { false; }; echo "c18: not reached" ); echo "c18: status $?"
( set -e; false; ) ; echo "c19: $?"
( set -e; [[ 1 -eq 2 ]]; echo "c20: [[ fails -> exit" ); echo "c20: status $?"
( set -e; (( 0 )); echo "c21: not reached" ); echo "c21: status $?"
( set -e; v=0; (( v++ )); echo "c22: not reached" ); echo "c22: status $?"
( set -e; true && false; echo "c23: last in && list fails" ); echo "c23: status $?"
( set -e; false && true; echo "c24: first in && list ok" ); echo "c24: status $?"
( set -e; set +e; false; echo "c25: disabled" )
( set -e; eval false; echo "c26: not reached" ); echo "c26: status $?"
( set -e; for i in 1; do false; done; echo "c27: not reached" ); echo "c27: status $?"
( set -e; case x in x) false;; esac; echo "c28: not reached" ); echo "c28: status $?"
( set -e; g() { return 3; }; g; echo "c29: not reached" ); echo "c29: status $?"
( set -e; (exit 5) || true; echo "c30: rescued" )
( set -e; x=$(false) || echo "c31: assignment rescued"; echo "c31: continues" )
( set -e; false; true ) && echo "c32: set -e ignored inside subshell on left of &&" || echo "c32: other"
( set -e; f33() { false; echo "c33: runs because caller is in && context"; }; f33 && echo "c33: and" )
( set -e; if (false; echo "c34: subshell in if condition continues"); then echo "c34: true"; fi )
( set -e; ! (false); echo "c35: negated subshell ok" )
( set -e; v=$(echo ok); echo "c36: $v" )
( set -e; readonly ro=1; echo "c37: before" )
( set -e; [ -f /nonexistent_file_zz ] && echo never; echo "c38: test in && list ok" )
( set -e; [ -f /nonexistent_file_zz ]; echo "c39: not reached" ); echo "c39: status $?"
echo "c40: main before"
set -e
true
echo "c41: about to fail"
false
echo "c42: never printed"
