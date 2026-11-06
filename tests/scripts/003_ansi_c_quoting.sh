# $'...' ANSI-C quoting
echo $'c1: tab\there'
echo $'c2: newline\nsecond'
echo $'c3: single \' quote'
echo $'c4: backslash \\ done'
echo $'c5: hex \x41\x42 octal \101\102'
echo $'c6: esc \e end' | cat -v
printf '%s\n' $'c7: \a\b\f\v\r' | cat -v
echo $'c8: double " quote and \" escaped'
echo $'c9: control \cA\cZ' | cat -v
echo $'c10: HI'
echo "c11: "$'mix\t'"ed"
x=$'a\nb'
echo "c12: ${#x}"
printf 'c13: [%s]\n' $'\x2a'
echo $'c14: unknown \q escape'
echo $'c15: \x4a\x4Bz'
echo $'c16: \1010'
echo $'c17: \?'
echo c18: $'\''
echo "c19: \$'not ansi in dquotes'" "$'x'"
v=$'trailing\n'
echo "c20: [${v}]"
echo $'c21: dollar \$ literal'
echo $'c22: \E' | cat -v
y=$'a b'
set -- $y; echo "c23: $#"
z=$'*'
echo "c24: [$z]"
echo c25: $'one'$'two' 'three'$'\x21'
echo $'c26: \x' | cat -v
case $'a\tb' in $'a\tb') echo "c27: case with ansi pattern";; esac
[[ $'x\ny' == x?y ]] && echo "c28: newline matches ?"
arr=($'a\tb' $'c d')
echo "c29: ${#arr[@]}"
echo $'c30: \0101'
echo $'c31: \x7e \176'
