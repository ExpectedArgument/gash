# printf: %q %b -v and escapes
printf 'c1: %q\n' "a b" "it's" 'a$b' '' 'plain' 'a"b' 'x*y'
printf 'c2: %q\n' $'tab\there' $'new\nline'
printf 'c3: %b\n' 'a\tb' 'c\\nd' '\x41\0101' 'no-escape'
printf 'c4: %b|\n' 'stop\chere' 'never printed'
printf 'c5: after\n'
printf -v var 'formatted %03d' 7
echo "c6: [$var]"
printf -v 'arr[1]' '%s' elem
echo "c7: ${arr[1]}"
printf -v multi '%s,' a b c
echo "c8: $multi"
printf -v empty ''
echo "c9: [$empty]"
printf '%s\n' 'c10: %s and \n in args are literal'
fmt='c11: %s-%s\n'
printf "$fmt" x y
printf 'c12: %-8s|%8s|\n' name value key 42
printf 'c13: %s\n' "$(printf '%.0s-' {1..10})"
printf 'c14: %3d%%\n' 50
printf 'c15: %s\n' {a..c}
printf 'c16: \\ \a' | cat -v; echo
printf 'c17: %b\n' '\e[1m' | cat -v
printf 'c18: %q\n' '~home' '#hash' 'semi;colon' 'back\slash'
printf 'c19: %5.1s|\n' xyz
printf 'c20: [%s]\n' "$(printf '%b' 'a\nb')"
printf 'c21: %s %b\n' '\t' '\t'
printf 'c22: %#.3g\n' 1
printf 'c23: %.3d\n' 7
printf 'c24: %-+5d|\n' 3
printf 'c25: [%c]\n' '' | cat -v
printf 'c26: %s\n' -v
x=$(printf '%s\n' a b c); echo "c27: ${#x}"
printf 'c28: %08.3f\n' -3.14159
printf 'c29: %i|%5i\n' 017 0x10
printf 'c30: %q\n' "$(printf 'a\x01b')"
printf 'c31: %b\n' '\0101' '\101' 'A'
printf 'c32: %q %q\n' 'a b' "c'd" | cat
printf -v q '%q' "it's here"; eval "r=$q"; echo "c33: $r"
printf 'c34: %s\n' "$(printf '%q' 'a|b&c<d>e(f)g')"
printf 'c35: %b' 'no newline\n'
printf 'c36: \101\102 \x43\n'
printf 'c37: %s\n' '%d'
printf '%s' 'c38: ' ; printf '%s\n' "end"
printf -v pad '%*s' 5 ''; echo "c39: [${pad}]"
printf -v line '%*s' 10 ''; echo "c40: ${line// /-}"
printf 'c41: %q\n' '[brackets]' '{braces}' '!bang'
printf 'c42: %5b|\n' 'ab'
printf 'c43: %c\n' 7 x
v=$(printf 'c44: %s' "trailing\n"); echo "$v"
