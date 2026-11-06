# while/until loops, break/continue with levels
i=0
while [ $i -lt 3 ]; do echo "c1: $i"; i=$((i+1)); done
i=0
until [ $i -ge 3 ]; do echo "c2: $i"; ((i++)); done
i=0
while true; do ((i++)); if ((i==2)); then continue; fi; if ((i>4)); then break; fi; echo "c3: $i"; done
for o in 1 2 3; do
  for n in a b c; do
    if [ $n = b ]; then continue 2; fi
    echo "c4: $o$n"
  done
done
for o in 1 2 3; do
  for n in a b c; do
    if [ $o$n = 2b ]; then break 2; fi
    echo "c5: $o$n"
  done
done
i=0
while ((i < 3)); do
  j=0
  while ((j < 3)); do
    ((j++))
    ((j == 2)) && continue
    ((i == 1)) && break
    echo "c6: $i.$j"
  done
  ((i++))
done
n=0; while false; do n=1; done; echo "c7: never ran $n status $?"
until true; do :; done; echo "c8: status $?"
i=0; while ((i++ < 3)); do :; done; echo "c9: i=$i"
k=3; while ((k--)); do echo "c10: k=$k"; done
cnt=0
while read -r w; do cnt=$((cnt+1)); done <<< $'a\nb\nc'
echo "c11: $cnt"
i=0; while :; do ((i++ >= 2)) && break; done; echo "c12: $i"
x=$(while true; do echo "c13: once"; break; done); echo "$x"
i=0; while [ $i -lt 2 ]; do i=$((i+1)); done > /dev/null; echo "c14: $i"
for i in 1 2 3; do (( i == 2 )) && break 5; echo "c15: $i"; done
f() { while true; do return 7; done; }; f; echo "c16: return from loop $?"
i=0; while true; do ((i++)); [ $i -ge 3 ] && { echo "c17: brace break"; break; }; done
i=0; until (( i == 2 )); do ((i++)); echo "c18: $i"; done
last=$(for i in 1 2 3; do echo $i; done | tail -n 1); echo "c19: $last"
while false; do :; done; echo "c20: $?"
i=0; while ((i < 2)); do ((i++)); (exit 3); done; echo "c21: loop status is last body status $?"
for i in 1 2 3; do
  case $i in 2) continue;; esac
  echo "c22: $i"
done
outer=0
while ((outer < 2)); do
  ((outer++))
  for x in a b; do
    until false; do
      echo "c23: $outer$x"
      continue 3
    done
  done
done
i=0
while
  ((i++))
  ((i < 3))
do
  echo "c24: multi-command condition $i"
done
for i in 1 2; do break; done; echo "c25: break status $?"
i=0; while true; do ((i++ > 1)) && break; echo "c26: $i"; done
s=""; for a in x y; do for b in 1 2; do s+="$a$b "; done; done; echo "c27: $s"
i=5; while ((i)); do ((i -= 2)); ((i < 0)) && break; done; echo "c28: $i"
