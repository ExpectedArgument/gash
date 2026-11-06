# single quotes, double quotes, concatenation, empty strings
x="hello world"
echo 'c1: $x not expanded'
echo "c2: $x expanded"
echo "c3: 'single inside double' $x"
echo 'c4: "double inside single"'
echo "c5: escaped \$x \"quote\" \\ backslash \` tick"
echo "c6: backslash before other \a \q stays"
echo 'c7: backslash in single \n \\ stays'
echo c8: ab'cd'"ef"gh
echo "c9: [${x}]" '[${x}]'
e=""
set -- $e; echo "c10: unquoted empty -> $# args"
set -- "$e"; echo "c11: quoted empty -> $# args"
set -- '' ""; echo "c12: two empty literals -> $# args"
set -- $e$e "a"$e; echo "c13: $#"
echo "c14: it's" 'it'\''s' "it's"
echo "c15: "'mixed'" concat"
y='*'
echo "c16: [$y]"
echo c17: "$x"'$x'$'\t'"|"
echo "c18: multi
line string"
echo 'c19: single
multi'
z="a  b   c"
echo c20: $z
echo "c21: $z"
echo "c22: ${#e} ${#x}"
echo c23: \"escaped\ space\"
echo c24: a\
b
echo "c25: dollar at end $"
echo "c26: \$(not cmd) and \`not\`"
echo "c27: tilde ~ in quotes" '~'
echo c28: "~"
echo "c29: $"'x' "\$"x
set -- "" ""; echo "c30: $# [$1$2]"
w="  padded  "
echo "c31: [$w]" [$w]
echo "c32: "''""''"done"
echo 'c33: '"'"'nested single via double'"'"
echo "c34: ${x:+"inner quoted"}" "${x:+'single in alt'}"
echo "c35: ${unset_v:-"a  b"}|"
echo "c36: "\""escaped dq outside"\"
echo "c37: $x"suffix "pre$x"
echo c38: "$"
echo c39: '$'"$"
a="it's \"quoted\""
echo "c40: $a"
