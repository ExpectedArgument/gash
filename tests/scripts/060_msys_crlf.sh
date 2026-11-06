# Git Bash (MSYS2) specific CR handling; v() shows CR as ^M
# Command substitution strips trailing "\r\n" (not only "\n"), so output of
# Windows programs captured with $(...) loses its trailing CRLF.
v() { local s=${2//$'\r'/^M}; echo "$1: [$s] len=${#2}"; }
a=$(printf 'a\r\n'); v c1 "$a"
a=$(printf 'a\r\n\r\n'); v c2 "$a"
a=$(printf 'a\r\nb\r\n'); v c3 "$a"
a=$(printf 'a\r'); v c4 "$a"
a=$(printf 'a\r\r\n'); v c5 "$a"
a=$(printf 'a\n\r\n'); v c6 "$a"
a=`printf 'a\r\n'`; v c7 "$a"
a="$(printf 'a\r\n')"; v c8 "$a"
set -- $(printf 'x\r\ny\r\n'); v c9 "$1"; v c9b "$2"
a=$(printf 'a\r\n' | cat); v c10 "$a"
# read and mapfile do NOT strip CR
read -r l < <(printf 'r\r\n'); v c11 "$l"
printf 'p\r\n' | { read -r l; v c12 "$l"; }
mapfile -t m < <(printf 'm1\r\nm2\r\n'); v c13 "${m[0]}"; v c13b "${m[1]}"
while IFS= read -r l; do v c14 "$l"; done < <(printf 'w1\r\nw2\r\n')
# here-string of a $(...) value: the value already lost its CR
read -r l <<< "$(printf 'h\r\n')"; v c15 "$l"
# a CR held in a variable survives inside $(...)
cr=$'\r'
a=$(printf '%s' "$cr"); v c16 "$a"
a=$(echo "x$cr"); v c17 "$a"
a=$(printf 'x%s\n' "$cr"); v c18 "$a"
# CR is not in IFS, so it stays attached to words
s="one$cr two$cr"
set -- $s; v c19 "$1"; v c19b "$2"
# trimming CR explicitly, the usual portable idiom
t="line$cr"; v c20 "${t%$'\r'}"
t2=$(printf 'k=v\r\n' | tr -d '\r'); v c21 "$t2"
