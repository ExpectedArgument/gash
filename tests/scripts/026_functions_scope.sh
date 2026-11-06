# local variables and dynamic scoping
x=global
show_x() { echo "$1: x=$x"; }
f1() { local x=f1local; show_x c1; f2; show_x c3; }
f2() { x=f2modified; show_x c2; }
f1
show_x c4
g() { local y; echo "c5: [${y-unset}] [${y:-empty}]"; y=set; }
g
echo "c6: y outside [${y-unset}]"
h() { x=changed_global; }
h; echo "c7: $x"
cnt=0
inc() { cnt=$((cnt+1)); }
inc; inc; echo "c8: $cnt"
outer() { local v=outer; middle; echo "c10: after middle v=$v"; }
middle() { echo "c9: middle sees v=$v"; v=middle_set; }
outer
echo "c11: global v=[${v-unset}]"
loc_arr() { local -a a=(1 2 3); echo "c12: ${#a[@]}"; }
loc_arr; echo "c13: [${a[*]-unset}]"
z=1
shadow() { local z=2; inner_shadow; echo "c15: $z"; }
inner_shadow() { local z=3; echo "c14: $z"; }
shadow; echo "c16: $z"
r() { local d=$1; if ((d > 0)); then r $((d-1)); fi; echo "c17: depth $d ${#FUNCNAME[@]}"; }
r 3
st() { local v=$(false); echo "c18: local masks status $?"; local w; w=$(false); echo "c19: separate assign $?"; }
st
p() { local IFS=,; set -- a b c; echo "c20: $*"; }
p; set -- a b c; echo "c21: $*"
dec() { declare inner_d=1; }
dec; echo "c22: declare in function is local [${inner_d-unset}]"
glob() { declare -g gvar=globalized; }
glob; echo "c23: $gvar"
q() { local x; x=local_q; unset x; echo "c24: after unset local [${x-unset}]"; }
x=gx; q
uq() { local x=lx; inner_unset; echo "c25: in uq after inner unset [${x-unset}]"; }
inner_unset() { unset x; }
x=gx; uq; echo "c26: global x=$x"
re() { local -r ro=1; ro=2; echo "c27: still running $ro"; }
( re ) 2>/dev/null; echo "c27: status $?"
f_args() { set -- x y; echo "c28: $#"; }
set -- 1 2 3; f_args; echo "c29: $#"
lv() { local a=1 b=2 c; c=$((a+b)); echo "c30: $a $b $c"; }
lv
mod_param() { local n=$1; n=$((n*2)); echo "c31: $n"; }
k=5; mod_param $k; echo "c32: k still $k"
local_same() { local x=$x; x+=_suffix; echo "c33: $x"; }
x=base; local_same; echo "c34: $x"
nested_local() { local v=1; ( v=2; echo "c35: subshell $v" ); echo "c36: $v"; }
nested_local
counter() { local -i c=0; c+=5; c+=5; echo "c37: $c"; }
counter
( local x=1 ) 2>/dev/null; echo "c38: local outside function status $?"
setglobal() { local tmp=1; G1=from_func; }
setglobal; echo "c39: $G1 [${tmp-unset}]"
order() { echo "c40: sees later global [$later]"; }
later=defined_after; order
