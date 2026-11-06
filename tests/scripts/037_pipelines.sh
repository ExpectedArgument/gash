# pipelines: exit status, PIPESTATUS, pipefail, negation, subshell isolation of variables
true | false; echo "c1: $?"
false | true; echo "c2: $?"
true | false | true; echo "c3: ${PIPESTATUS[*]}"
false | true | (exit 3); echo "c4: ${PIPESTATUS[@]}"
(exit 2) | (exit 5); s=("${PIPESTATUS[@]}"); echo "c5: ${s[0]} ${s[1]} ${#s[@]}"
true; echo "c6: ${PIPESTATUS[*]}"
false; echo "c7: ${PIPESTATUS[*]}"
set -o pipefail
false | true; echo "c8: pipefail $?"
true | (exit 4) | true; echo "c9: $?"
(exit 2) | (exit 3) | true; echo "c10: rightmost failing $?"
true | true; echo "c11: $?"
set +o pipefail
false | true; echo "c12: $?"
! true; echo "c13: $?"
! false; echo "c14: $?"
! true | false; echo "c15: $?"
! (exit 7); echo "c16: $?"
x=outer
echo val | x=inner; echo "c17: $x"
echo val | read y; echo "c18: [${y-unset}]"
echo piped | { read -r z; echo "c19: inside [$z]"; }
c=0; printf '1\n2\n3\n' | while read -r n; do c=$((c+n)); done; echo "c20: $c"
c=$(printf '1\n2\n3\n' | { s=0; while read -r n; do s=$((s+n)); done; echo $s; }); echo "c21: $c"
