# indexed arrays: sparse, +=, unset, slicing, quoting
a=(zero one two)
echo "c1: ${a[0]} ${a[2]} ${#a[@]}"
a+=(three "four five")
echo "c2: ${#a[@]} [${a[4]}]"
a[10]=ten
echo "c3: ${#a[@]} ${!a[*]}"
unset 'a[1]'
echo "c4: ${#a[@]} ${!a[*]}"
a+=(eleven)
echo "c5: ${!a[*]}"
echo "c6: ${a[-1]} ${a[-2]}"
printf 'c7: [%s]\n' "${a[@]}"
b=("${a[@]}")
echo "c8: ${!b[*]}"
c=()
c[3]=x
c+=(y)
echo "c9: ${!c[@]} ${c[*]}"
d=([2]=two [0]=zero)
echo "c10: ${!d[@]} ${d[@]}"
e=(x [5]=five y)
echo "c11: ${!e[@]}"
s="a b c"
f=($s)
echo "c12: ${#f[@]}"
g=("$s")
echo "c13: ${#g[@]}"
h=(one two three)
h[1]+=_appended
echo "c14: ${h[*]}"
unset h
echo "c15: [${h[*]-gone}] ${#h[@]}"
j=(3 1 2)
sorted=($(printf '%s\n' "${j[@]}" | sort))
echo "c16: ${sorted[*]}"
k=(a b c)
k=("${k[@]:0:1}" X "${k[@]:1}")
echo "c17: ${k[*]}"
l=(a b c d)
unset 'l[1]'; l=("${l[@]}")
echo "c18: ${!l[*]} ${l[*]}"
m=(1 2 3)
for i in "${!m[@]}"; do m[i]=$(( m[i] * 10 )); done
echo "c19: ${m[*]}"
declare -a n
echo "c20: ${#n[@]}"
n[0]=first
echo "c21: $n ${n}"
o=x
o[1]=y
echo "c22: ${o[*]} ${#o[@]}"
p=(1 2 3)
p=()
echo "c23: ${#p[@]}"
q=([0]=a [1]=b)
echo "c24: ${q[@]: -1}"
r=( $(echo "w1 w2") "lit" )
echo "c25: ${#r[@]}"
str=hello
echo "c26: ${str[0]} ${#str[@]}"
t=(a b c)
echo "c27: ${t[@]:1} ${t[*]:0:2}"
idx=1
echo "c28: ${t[idx+1]} ${t[$idx]}"
u=(x y z)
echo "c29: $(( ${#u[@]} - 1 ))"
contains() { local n=$1; shift; local e; for e; do [[ $e == "$n" ]] && return 0; done; return 1; }
contains b "${t[@]}" && echo "c30: contains b"
contains q "${t[@]}" || echo "c31: no q"
v=( "${t[@]/b/B}" )
echo "c32: ${v[*]}"
w=("a b" "c d")
w2=(${w[@]})
echo "c33: ${#w2[@]}"
x=(1 2 3)
x[1]=
echo "c34: ${#x[@]} [${x[1]}]"
unset 'x[-1]'
echo "c35: ${!x[*]}"
y=(); y[5]=five; y[2]=two
echo "c36: ${y[*]} | ${!y[*]}"
z=(a b c d e f)
echo "c37: ${z[@]:2:3} | ${z[@]: -3:2}"
stack=(); stack+=(1); stack+=(2); stack+=(3)
top=${stack[-1]}; unset 'stack[-1]'
echo "c38: popped $top, left ${stack[*]}"
queue=(a b c); first=${queue[0]}; queue=("${queue[@]:1}")
echo "c39: dequeued $first, left ${queue[*]} (${#queue[@]})"
mat=(); for i in 0 1; do for j in 0 1; do mat[i*2+j]=$i$j; done; done
echo "c40: ${mat[*]}"
rev=(); for ((i=${#t[@]}-1; i>=0; i--)); do rev+=("${t[i]}"); done
echo "c41: ${rev[*]}"
words=(the quick brown fox)
max=""; for wd in "${words[@]}"; do (( ${#wd} > ${#max} )) && max=$wd; done
echo "c42: longest=$max"
copy=("${words[@]}"); copy[0]=THE
echo "c43: ${words[0]} ${copy[0]}"
