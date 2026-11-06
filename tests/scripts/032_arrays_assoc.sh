# associative arrays (key order is hash-based, so output is sorted)
declare -A h
h[one]=1
h[two]=2
h["key with spaces"]="val with spaces"
echo "c1: ${h[one]} ${h[two]} [${h[key with spaces]}]"
echo "c2: ${#h[@]}"
for k in "${!h[@]}"; do echo "c3: [$k]=[${h[$k]}]"; done | sort
printf '%s\n' "${h[@]}" | sort | sed 's/^/c4: /'
[[ -v h[one] ]]; echo "c5: $?"
[[ -v h[nope] ]]; echo "c6: $?"
[[ -v "h[key with spaces]" ]]; echo "c7: $?"
unset 'h[one]'
echo "c8: ${#h[@]} [${h[one]-unset}]"
h+=([three]=3 [four]=4)
echo "c9: ${#h[@]} ${h[three]}"
h[two]+=more
echo "c10: ${h[two]}"
declare -A m=([a]=1 [b]=2 [c]=3)
sum=0; for k in "${!m[@]}"; do (( sum += m[$k] )); done; echo "c11: $sum"
k="dynamic key"
m[$k]=dyn
echo "c12: ${m[$k]} ${m["dynamic key"]}"
declare -A empty
echo "c13: ${#empty[@]}"
declare -A counts
for w in apple banana apple cherry banana apple; do counts[$w]=$(( ${counts[$w]:-0} + 1 )); done
for w in "${!counts[@]}"; do echo "c14: $w ${counts[$w]}"; done | sort
declare -A num
num[1]=one; num[01]=zero-one
echo "c15: ${num[1]} ${num[01]} ${#num[@]}"
i=1
num[i]=letter-i
echo "c16: ${num[i]} ${num[$i]}"
dump() { local -n src=$1; local k; for k in "${!src[@]}"; do echo "$k=${src[$k]}"; done | sort; }
dump m | sed 's/^/c17: /'
unset m
declare -A m
echo "c18: ${#m[@]}"
declare -A q=(["a b"]=x [c]="y z")
echo "c19: [${q["a b"]}] [${q[c]}]"
declare -A one=([only]=1)
declare -p one | sed 's/^/c20: /'
f() { declare -A loc=([x]=1); echo "c21: ${loc[x]}"; }
f; echo "c22: [${loc[x]-none}]"
declare -A sp
sp[" "]=space
echo "c23: [${sp[" "]}] ${#sp[@]}"
declare -A kv=([x]=1 [y]=2)
keys=$(printf '%s\n' "${!kv[@]}" | sort | tr '\n' ' ')
echo "c24: $keys"
for k in $(printf '%s\n' "${!kv[@]}" | sort); do echo "c25: $k -> ${kv[$k]}"; done
declare -A nested
nested[a.b]=dot; nested[a/b]=slash; nested['$x']=dollar
echo "c26: ${nested[a.b]} ${nested[a/b]} ${nested['$x']}"
declare -A ar2=([k1]=v1)
ar2[k1]=replaced
echo "c27: ${ar2[k1]} ${#ar2[@]}"
[[ ${ar2[missing]+set} ]] || echo "c28: +set idiom says missing"
[[ ${ar2[k1]+set} ]] && echo "c29: +set idiom says present"
declare -A cnt2=()
(( cnt2[x]++ )); (( cnt2[x]++ )); (( cnt2[y] += 5 ))
echo "c30: ${cnt2[x]} ${cnt2[y]}"
declare -A spaces=(["a  b"]=double)
echo "c31: [${spaces["a  b"]}] [${spaces[a  b]}]"
key='a  b'
echo "c32: [${spaces[$key]}]"
declare -A fromloop
for pair in name=alice age=30 city=paris; do fromloop[${pair%%=*}]=${pair#*=}; done
echo "c33: ${fromloop[name]} ${fromloop[age]} ${fromloop[city]}"
unset 'fromloop[age]'
echo "c34: ${#fromloop[@]}"
