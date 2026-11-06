# ${x:-} ${x-} := :? :+ and friends
unset u; e=""; s="set"
echo "c1: [${u:-def}] [${e:-def}] [${s:-def}]"
echo "c2: [${u-def}] [${e-def}] [${s-def}]"
echo "c3: [${u:+alt}] [${e:+alt}] [${s:+alt}]"
echo "c4: [${u+alt}] [${e+alt}] [${s+alt}]"
echo "c5: [${u:=assigned}] u=[$u]"
unset u
e=""
echo "c6: [${e:=filled}] e=[$e]"
e=""
echo "c7: [${e=notfilled}] e=[$e]"
unset u
echo "c8: [${u=eq}] u=[$u]"
( unset v; echo "${v:?custom message}"; echo "c9: not reached" ) 2>/dev/null
echo "c9: status $?"
( v=""; : "${v:?}"; echo "c10: not reached" ) 2>/dev/null
echo "c10: status $?"
( v=""; : "${v?}"; echo "c11: reached (empty is set)" ) 2>/dev/null
echo "c11: status $?"
( v="ok"; echo "c12: ${v:?nope}" ) 2>/dev/null
echo "c12: status $?"
echo "c13: ${u:-$(echo nested)} ${unset2:-"quoted default"}"
d="dval"
echo "c14: ${unset3:-$d} ${unset3:-${d}x}"
echo "c15: ${unset4:-a b   c}"
set -- ${unset5:-x y z}; echo "c16: $#"
set -- "${unset5:-x y z}"; echo "c17: $#"
echo "c18: ${unset6:-'single quoted'}"
echo "c19: ${unset7:-\$literal}"
echo "c20: ${s:+"alt with space"}"
n=0
echo "c21: ${n:-zero-is-set}"
echo "c22: ${1:-no-arg1} ${#}"
set -- first
echo "c23: ${1:-no-arg1} ${2-no-arg2}"
arr=()
echo "c24: ${arr[0]:-empty-array}"
: ${init:=initialized}
echo "c25: $init"
echo "c26: ${unset8:-${unset9:-deep}}"
( : ${1:?} ) 2>/dev/null; echo "c27: positional :? status $?"
( : ${2:?} ) 2>/dev/null; echo "c28: positional :? status $?"
( : ${unset10:=x}; echo "c29: [$unset10]" )
echo "c30: [${unset10-still unset in parent}]"
arr2=(a b)
echo "c31: ${arr2[5]:-oob} ${arr2[1]:-x}"
: ${arr2[3]:=assigned3}
echo "c32: ${arr2[3]} ${#arr2[@]}"
echo "c33: ${s:+${d:+both}}"
f() { local loc; echo "c34: ${loc:-local default} ${loc-local unset}"; }
f
echo "c35: [${e:+}] [${s:+}] [${u:-}]"
x=5
echo "c36: $(( ${x:-0} + ${nothing:-10} ))"
echo "c37: ${unset11:-*}"
