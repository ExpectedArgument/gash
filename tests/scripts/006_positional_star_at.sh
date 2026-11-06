# "$@" vs $@ vs "$*" vs $* with IFS changes
show() { local l=$1; shift; local s="$l: $#:"; local a; for a in "$@"; do s+=" [$a]"; done; echo "$s"; }
set -- "a b" c "" "d  e"
show c1 "$@"
show c2 $@
show c3 "$*"
show c4 $*
IFS=,
show c5 "$*"
show c6 "$@"
show c7 $*
show c8 $@
IFS=:-
show c9 "$*"
IFS=
show c10 "$*"
show c11 $*
show c12 $@
unset IFS
show c13 "$*"
show c14 $*
IFS=$' \t\n'
show c15 "${@:2}"
show c16 "${@:2:2}"
show c17 "${*:2}"
show c18 "${@: -1}"
show c19 "${@:1:1}x"
f() { show c20 "$@"; echo "c21: $# [$1] [${10}] [${11}]"; }
f 1 2 3 4 5 6 7 8 9 ten eleven
set --
show c22 "$@"
show c23 "$*"
show c24 "${@}x"
show c25 $@
set -- one
show c26 "x${@}y"
set -- a b
show c27 "x${@}y"
show c28 "pre$*post"
show c29 "pre$@post"
set -- "x y"
show c30 $@
IFS=x
show c31 $@
show c32 "$*"
unset IFS
set -- a b c
IFS=,
joined="$*"
unset IFS
echo "c33: $joined"
arr=("p q" r)
IFS=+
show c34 "${arr[*]}"
show c35 ${arr[*]}
show c36 "${arr[@]}"
unset IFS
g() { echo "c37: $#"; }
set -- "" "" ""
g "$@"
g $@
g "$*"
g $*
