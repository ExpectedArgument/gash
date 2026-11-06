# read builtin: -r -a -d -n -N, IFS splitting, loops
read a b c <<< "1 2 3 4 5"
echo "c1: [$a] [$b] [$c]"
read a b c <<< "only"
echo "c2: [$a] [$b] [$c]"
read -r x <<< 'back\slash'
echo "c3: [$x]"
read x <<< 'back\slash'
echo "c4: [$x]"
read x <<< 'trailing\'
echo "c5: [$x] $?"
printf 'line\\\ncontinued\n' | { read x; echo "c6: [$x]"; }
printf 'line\\\ncontinued\n' | { read -r x; echo "c7: [$x]"; }
IFS=: read -r u v w <<< "a:b:c:d"
echo "c8: [$u] [$v] [$w]"
IFS=, read -r p q <<< "1,2,"
echo "c9: [$p] [$q]"
IFS=, read -r p q <<< "1,2,3,"
echo "c10: [$p] [$q]"
IFS=, read -r p q <<< ",x"
echo "c11: [$p] [$q]"
read -r s <<< "   lead and trail   "
echo "c12: [$s]"
IFS= read -r s <<< "   lead and trail   "
echo "c13: [$s]"
read -r <<< "  reply var  "
echo "c14: [$REPLY]"
read -ra arr <<< "x y   z"
echo "c15: ${#arr[@]} ${arr[2]}"
IFS=: read -ra parts <<< "a::b"
echo "c16: ${#parts[@]} [${parts[1]}]"
read -r -d ';' t <<< "first;second"
echo "c17: [$t] $?"
read -r -d '' all <<< "no delim"
echo "c18: status $? [$all]"
read -r -n 3 n3 <<< "abcdef"
echo "c19: [$n3]"
read -r -n 10 n10 <<< "ab"
echo "c20: [$n10]"
read -r -N 4 nn <<< "ab cd"
echo "c21: [$nn]"
while IFS= read -r line; do echo "c22: [$line]"; done <<< $'  indented\n\ttabbed\nlast'
printf 'a\nb' | { while read -r l; do echo "c23: $l"; done; echo "c23: leftover [$l]"; }
printf 'a\nb' | { while read -r l || [[ -n $l ]]; do echo "c24: $l"; done; }
read -r e < /dev/null; echo "c25: eof status $? [$e]"
echo "x y" | { read -r f1 f2; echo "c26: $f2 $f1"; }
read -r a1 a2 <<< ""
echo "c27: [$a1] [$a2]"
IFS=$'\t' read -r c1 c2 c3 <<< $'a\t\tc'
echo "c28: [$c1] [$c2] [$c3]"
IFS=' ' read -r c1 c2 c3 <<< 'a  c'
echo "c29: [$c1] [$c2] [$c3]"
read -r -a empty_arr <<< ""
echo "c30: ${#empty_arr[@]}"
while read -r k v; do echo "c31: $k => $v"; done <<'EOF'
alpha 1
beta two words
EOF
read -r -d $'\n' z <<< "nl delim"; echo "c32: [$z]"
IFS= read -r -d '' -a nul_arr < <(printf 'a\0b\0')
echo "c33: [${nul_arr[0]}]"
while IFS= read -r -d '' item; do echo "c34: [$item]"; done < <(printf 'p q\0r\0')
IFS=, read -r -a csv <<< "a,b,,d"
echo "c35: ${#csv[@]} [${csv[2]}] [${csv[3]}]"
read -r one <<< $'first\nsecond'
echo "c36: [$one]"
{ read -r l1; read -r l2; } <<< $'L1\nL2\nL3'
echo "c37: $l1 $l2"
IFS=: read -r user _ uid _ <<< "root:x:0:0:root:/root:/bin/bash"
echo "c38: $user $uid"
read -r -n 1 ch <<< "xyz"; echo "c39: [$ch]"
read -r -d , f1 <<< "a b,c"; echo "c40: [$f1]"
IFS= read -r -n 4 sp <<< "  ab"; echo "c41: [$sp]"
read -r -a words <<< "  multiple   spaces  here  "; echo "c42: ${#words[@]} [${words[*]}]"
read v1 <<< 'esc\aped\\x'; echo "c43: [$v1]"
n=0; while read -r; do ((n++)); done <<< $'a\nb\n\nc'; echo "c44: $n lines"
IFS=';' read -r -a semi <<< "x;y;z"; echo "c45: ${semi[1]} ${#semi[@]}"
