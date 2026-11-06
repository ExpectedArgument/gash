# command substitution, part 1: $(), backticks, nesting, newline stripping, exit status
x=$(echo hello)
echo "c1: $x"
y=`echo backtick`
echo "c2: $y"
z=$(printf 'trail\n\n\n')
echo "c3: [$z]"
w=$(printf '\n\nlead')
echo "c4: [$w]"
m=$(printf 'a\nb\n')
echo "c5: [$m]"
echo "c6: $(echo $(echo $(echo nested)))"
echo "c7: `echo \`echo inner-backtick\``"
echo "c8: $(echo "quoted $(echo 'inner')")"
v=$(exit 3); echo "c9: $?"
v=$(false); echo "c10: $?"
v=$(true); echo "c11: $?"
v=$(echo out; exit 4); echo "c12: [$v] $?"
tmp=$(mktemp -d) || exit 1
cd "$tmp" || exit 1
printf 'file content\nline2\n' > f.txt
echo "c13: [$(< f.txt)]"
c=$(cat f.txt); echo "c14: ${#c}"
cd .. && rm -rf "$tmp"
echo "c15: $(echo a; echo b)"
echo c16: $(echo a; echo b)
n=$(echo 5); echo "c17: $((n * 2))"
echo "c18: $( echo "  spaced  " )|"
echo "c19: $(for i in 1 2 3; do printf '%s' "$i"; done)"
s='single $(not expanded)'; echo "c20: $s"
echo "c21: \$(literal)"
a=$(echo one) b=$(echo two); echo "c22: $a $b"
echo "c23: $(echo 'it'"'"'s')"
q=$(echo "a\\b"); echo "c24: $q"
r=`echo 'a\\b'`; echo "c25: $r"
t=`echo "x\$y"`; echo "c26: [$t]"
echo "c27: $( (( 1 + 1 )); echo $? )"
