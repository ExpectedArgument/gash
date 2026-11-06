# unset, declare -p, type -t, command -v, builtin/command
x=1
unset x; echo "c1: [${x-unset}]"
y=2; unset -v y; echo "c2: [${y-unset}]"
f() { :; }
unset -f f; type -t f >/dev/null; echo "c3: func unset status $?"
both=var
both() { echo func; }
unset both; echo "c4: [${both-unset}] $(type -t both)"
unset both; echo "c5: [$(type -t both)]"
unset nonexistent; echo "c6: unset missing status $?"
a=(1 2 3); unset 'a[1]'; echo "c7: ${a[*]} ${#a[@]}"
s="hello"
declare -p s
declare -i i=42; declare -p i
declare -a arr=(one "two words"); declare -p arr
declare -r ro=const; declare -p ro
declare -x ex=exported; declare -p ex
declare -l lo=ABC; declare -p lo
q='say "hi" $USER \ back`tick`'
declare -p q
declare -p undefined_var 2>/dev/null; echo "c8: declare -p missing $?"
declare -a sparse=([1]=a [5]=b); declare -p sparse
declare empty; declare -p empty
declare -a emptyarr=(); declare -p emptyarr
declare -A assoc1=([key]=val); declare -p assoc1
declare -ia intarr=(1 2); declare -p intarr
declare -rx rox=both; declare -p rox
echo "c9: $(type -t echo) $(type -t if) $(type -t cat) $(type -t declare) $(type -t [[) $(type -t test)"
myfunc() { :; }
shopt -s expand_aliases
alias myalias='echo aliased'
echo "c10: $(type -t myfunc) $(type -t myalias) [$(type -t no_such_thing)]"
type -t no_such_thing; echo "c11: status $?"
echo "c12: $(command -v echo) $(command -v if) $(command -v myfunc) $(command -v declare)"
command -v no_such_cmd; echo "c13: status $?"
echo "c14: $(command -v myalias)"
type echo | sed 's/^/c15: /'
type if | sed 's/^/c16: /'
type -t command | sed 's/^/c17: /'
echo() { builtin echo "wrapped: $*"; }
echo "c18: x"
command echo "c19: command bypasses function"
builtin echo "c20: builtin"
unset -f echo
echo "c21: restored"
declare -F myfunc; echo "c22: $?"
declare -F nofunc; echo "c23: $?"
type -P cat >/dev/null; echo "c24: type -P finds external $?"
type -a true | head -n 1 | sed 's/^/c25: /'
declare -i n; n="3 + 4"; echo "c26: $n"
declare +i n; n="3 + 4"; echo "c27: $n"
readonly rv=1; unset rv 2>/dev/null; echo "c28: unset readonly status $? $rv"
unset a; echo "c29: ${#a[@]}"
type myfunc | head -n 1 | sed 's/^/c30: /'
type myalias | sed 's/^/c31: /'
type -t ls_not_here_xyz || echo "c32: not found"
command -V echo | sed 's/^/c33: /'
v1=1 v2=2; unset v1 v2; echo "c34: [${v1-u}${v2-u}]"
declare -a nums=(3 1 2); unset 'nums[0]'; declare -p nums
unset -f no_such_func; echo "c35: $?"
declare -i calc=5*5; declare -p calc
declare -- dash_var="plain"; declare -p dash_var
nl=$'line1\nline2'; declare -p nl
tabv=$'a\tb'; declare -p tabv
declare -g gl=1; declare -p gl
