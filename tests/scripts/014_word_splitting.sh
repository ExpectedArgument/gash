# word splitting with default and custom IFS
show() { local l=$1; shift; local s="$l: $#:"; local a; for a in "$@"; do s+=" [$a]"; done; echo "$s"; }
v="  a  b   c  "
show c1 $v
show c2 "$v"
IFS=:
v="a:b::c:"
show c3 $v
v=":a:b"
show c4 $v
v="::"
show c5 $v
IFS=': '
v=" a : b  :: c "
show c6 $v
IFS=' '
v=$'a\tb\nc d'
show c7 $v
IFS=$'\n'
show c8 $v
IFS=
v="a b c"
show c9 $v
unset IFS
v=$'x\ty\nz'
show c10 $v
IFS=,
v="1,2,3"
show c11 $v "$v" ${v}x
w=one,two
show c12 $v$w
arr=($v)
show c13 "${arr[@]}"
x=a,b
show c14 pre${x}post
show c15 ${unset_var} "" ''
n=" "
show c16 $n
IFS=$' \t\n'
cmd=$(printf 'l1 w1\nl2 w2\n')
show c17 $cmd
show c18 "$cmd"
IFS=x
v=axxbxc
show c19 $v
read -r p q <<< "1x2x3"
show c20 "$p" "$q"
unset IFS
set -- $(echo "  spaced   out  ")
show c21 "$@"
IFS=' :'
v="a: b:c :d"
show c22 $v
v=" : "
show c23 $v
v="a:"
show c24 $v
v="a: "
show c25 $v
unset IFS
v="a"
show c26 $v$v "$v"$v
e=""
show c27 $e$e $e"" "$e"$e
IFS=,
show c28 a,b
v="a,b"
show c29 "x"$v"y"
unset IFS
f() { local IFS=/; show c30 $1; }
f "p/q/r"
show c31 $(echo "a/b")
IFS=$'\t'
v=$'a\t\tb'
show c32 $v
IFS=$'\t,'
v=$'a\t,b'
show c33 $v
unset IFS
v="1 2"
for i in $v; do show c34 "$i"; done
for i in "$v"; do show c35 "$i"; done
set -- $(( 1 + 2 ))$v
show c36 "$@"
