# functions: definition forms, recursion, return codes, FUNCNAME
greet() { echo "c1: hello $1"; }
greet world
function kw_style { echo "c2: function keyword $#"; }
kw_style a b
function both() { echo "c3: both forms"; }
both
fact() { if (( $1 <= 1 )); then echo 1; else echo $(( $1 * $(fact $(( $1 - 1 ))) )); fi; }
echo "c4: $(fact 10)"
fib() { local n=$1; if ((n < 2)); then REPLY=$n; return; fi; local a; fib $((n-1)); a=$REPLY; fib $((n-2)); REPLY=$((a+REPLY)); }
fib 15; echo "c5: $REPLY"
rc() { return $1; }
rc 0; echo "c6: $?"
rc 5; echo "c7: $?"
rc 255; echo "c8: $?"
rc 256; echo "c9: $?"
rc 300; echo "c10: $?"
lastst() { false; return; }
lastst; echo "c11: $?"
noreturn() { (exit 9); }
noreturn; echo "c12: $?"
args() { echo "c13: $# [$1] [$2] [$*]"; }
args "a b" c
args
set -- outer1 outer2
pos() { echo "c14: inside $1 $#"; }
pos inner
echo "c15: outside $1 $#"
outer() { inner() { echo "c16: inner defined at runtime"; }; }
type -t inner >/dev/null || echo "c17: inner not yet defined"
outer
inner
redefine() { echo "c18: v1"; }
redefine
redefine() { echo "c18: v2"; }
redefine
cnt() { echo $#; }
echo "c19: $(cnt "$@") $(cnt) $(cnt '' '')"
f_status() { return 3; }
if f_status; then echo no; else echo "c20: function in if false $?"; fi
f_status && echo no || echo "c21: in or-list"
multi() {
  echo "c22: line1"
  echo "c22: line2"
}
multi | while read -r l; do echo "piped: $l"; done
sub_body() ( x=inside; echo "c23: subshell body $x" )
x=outside
sub_body
echo "c24: x=$x"
names() { echo "c25: ${FUNCNAME[0]} ${FUNCNAME[1]-none} ${#FUNCNAME[@]}"; }
names
caller_fn() { names; }
caller_fn
echo "c26: [${FUNCNAME[*]}]"
f-dash() { echo "c27: dash in name"; }
f-dash
f.dot() { echo "c28: dot in name"; }; f.dot
early() { echo "c29: before return"; return 0; echo "never"; }
early
neg() { return -1; }
neg 2>/dev/null; echo "c30: return -1 gives $?"
cond_def() { echo "c31: conditional def"; }
if true; then cond2() { echo "c32: defined in if"; }; fi
cond_def; cond2
countdown() { local n=$1; ((n == 0)) && { echo "c33: liftoff"; return; }; echo "c33: $n"; countdown $((n-1)); }
countdown 3
deep() { local d=$1; ((d >= 50)) && { echo "c34: reached depth $d with ${#FUNCNAME[@]} frames"; return; }; deep $((d+1)); }
deep 0
ret_val() { local r=$(( $1 * 2 )); echo "$r"; return $(( $1 % 2 )); }
v=$(ret_val 21); echo "c35: $v $?"
compound() if [ "$1" = yes ]; then echo "c36: if as function body"; fi
compound yes
loopfn() for i in 1 2; do echo "c37: for body $i"; done
loopfn
brace_redirect() { echo "c38: body"; } >/dev/null
brace_redirect; echo "c38: redirect on definition hides output"
recurse_sum() { if (( $1 == 0 )); then echo 0; else echo $(( $1 + $(recurse_sum $(( $1 - 1 ))) )); fi; }
echo "c39: $(recurse_sum 10)"
