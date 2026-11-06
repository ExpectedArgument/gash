# for-in and C-style for loops
for x in a b c; do echo "c1: $x"; done
for x in "a b" c; do echo "c2: [$x]"; done
set -- p "q r"
for x; do echo "c3: [$x]"; done
for x in; do echo "c4: never"; done; echo "c4: empty list status $?"
for ((i=0; i<3; i++)); do echo "c5: $i"; done
for ((i=10, j=0; i>j; i-=3, j++)); do echo "c6: $i $j"; done
for ((;;)); do echo "c7: infinite once"; break; done
for (( k = 1 ; k <= 4 ; k *= 2 )); do echo "c8: $k"; done
echo "c9: after loop k=$k i=$i"
for x in {1..3}; do printf 'c10: %s\n' "$x"; done
arr=(one "two three" four)
for e in "${arr[@]}"; do echo "c11: [$e]"; done
for e in ${arr[@]}; do echo "c12: [$e]"; done
for i in ${!arr[@]}; do echo "c13: $i=${arr[i]}"; done
for x in a b; do :; done; echo "c14: x after loop = $x"
s="x y z"
for w in $s; do echo "c15: $w"; done
for w in "$s"; do echo "c16: $w"; done
for x in $(echo 1 2) 3; do echo "c17: $x"; done
for i in 1 2 3
do
  echo "c18: $i"
done
for i in 1 2; { echo "c19: brace body $i"; }
for ((i=0;i<2;i++)) { echo "c20: c-style brace $i"; }
for ((i=0; i<3; i++)); do :; done; echo "c21: $i"
for ((n=3; n--; )); do echo "c22: $n"; done
for x in a b c; do false; done; echo "c23: status $?"
for ((i=0; i<0; i++)); do :; done; echo "c24: zero iterations status $?"
for i in 3 2 1; do echo -n "c25:$i "; done; echo
f() { for a in "$@"; do echo "c26: [$a]"; done; }
f "x y" "" z
for v in '*' "?"; do echo "c27: [$v]"; done
for x in a b c; do
  for y in 1 2; do
    printf 'c28: %s%s\n' "$x" "$y"
  done
done | head -n 3
for ((i=1; i<=3; i++)); do
  for ((j=1; j<=i; j++)); do printf '*'; done
  echo " c29: row $i"
done
total=0; for n in 1 2 3 4 5; do (( total += n )); done; echo "c30: $total"
for x in one two; do echo "c31: $x"; done > /dev/null; echo "c31: redirected loop"
for i in 1 2 3; do [ $i -eq 2 ] && continue; echo "c32: $i"; done
for ((i=0, s=0; i<5; s+=i, i++)); do :; done; echo "c33: $s"
for in_kw in in do done; do echo "c34: $in_kw"; done
for ((i = 2 ** 3; i > 6; i--)); do echo "c35: $i"; done
