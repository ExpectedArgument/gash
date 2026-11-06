# readonly and export
readonly R=1
echo "c1: $R"
( R=2; echo "c2: after failed assignment" ) 2>/dev/null; echo "c2: status $?"
( R=2 ) 2>/dev/null; echo "c3: $?"
( readonly R=3 ) 2>/dev/null; echo "c4: $?"
( unset R ) 2>/dev/null; echo "c5: $?"
f() { local R=5; echo "c6: local shadow $R"; }
( f ) 2>/dev/null; echo "c6: status $?"
readonly -a RA=(x y)
( RA[0]=z ) 2>/dev/null; echo "c7: $? ${RA[0]}"
declare -r DR=dr
( DR=new ) 2>/dev/null; echo "c8: $?"
readonly NOVAL
( NOVAL=x ) 2>/dev/null; echo "c9: $?"
readonly -p | grep -E '^declare -r (R|DR)=' | sed 's/^/c10: /'
export E1=one
declare -p E1 | sed 's/^/c11: /'
export -n E1
declare -p E1 | sed 's/^/c12: /'
E2=two; export E2
export -p | grep 'E2=' | sed 's/^/c13: /'
export E3
declare -p E3 2>/dev/null | sed 's/^/c14: /'
E4=four; declare -x E4; declare -p E4 | sed 's/^/c15: /'
declare +x E4; declare -p E4 | sed 's/^/c16: /'
f2() { export FROM_FUNC=yes; }
f2; declare -p FROM_FUNC | sed 's/^/c17: /'
E5=five; ( export E5; declare -p E5 | sed 's/^/c18: /' ); declare -p E5 | sed 's/^/c19: /'
TEMP_ENV=temp declare -p TEMP_ENV 2>/dev/null | sed 's/^/c20: /'
g() { echo "c21: [$TEMPVAR]"; }
TEMPVAR=prefixed g
echo "c22: after [${TEMPVAR-unset}]"
TV2=x true; echo "c23: [${TV2-unset}]"
( R+=x ) 2>/dev/null; echo "c24: append to readonly $?"
( readonly R ) 2>/dev/null; echo "c25: re-readonly without value $?"
roloc() { local -r L=1; echo "c26: $L"; }
roloc; L=2; echo "c27: local readonly gone after return $L"
export E6="with space"; declare -p E6 | sed 's/^/c28: /'
export A1=1 A2=2; declare -p A1 A2 | sed 's/^/c29: /'
( R=3; echo "c30: unreachable?" ) 2>/dev/null; echo "c30: $?"
for v in R DR E1; do [[ -v $v ]] && echo "c31: $v defined"; done
