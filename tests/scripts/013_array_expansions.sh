# array parameter expansions: ${#a[@]} ${!a[@]} ${a[@]:o:l} "${a[*]}" patterns
a=(one "two words" three "" five)
echo "c1: ${#a[@]} ${#a[*]}"
echo "c2: ${!a[@]}"
echo "c3: ${a[@]:1:2}"
printf 'c4: [%s]\n' "${a[@]:1:2}"
printf 'c5: [%s]\n' "${a[@]}"
printf 'c6: [%s]\n' ${a[@]}
printf 'c7: [%s]\n' "${a[*]}"
IFS=-
printf 'c8: [%s]\n' "${a[*]}"
unset IFS
s=([3]=c [1]=a [7]=g)
echo "c9: ${!s[@]} | ${s[@]} | ${#s[@]}"
echo "c10: ${s[@]:2}"
echo "c11: ${s[@]:0:1}"
echo "c12: ${s[-1]} ${s[7]} [${s[5]}]"
echo "c13: ${a[@]: -2:1}"
printf 'c14: [%s]\n' "${a[@]/#/<}"
printf 'c15: [%s]\n' "${a[@]%s}"
e=()
echo "c16: ${#e[@]} [${e[@]}] [${!e[@]}]"
printf 'c17: [%s]\n' "${e[@]}"
set -- "${e[@]}"; echo "c18: $#"
i=2
echo "c19: ${a[i]} ${a[$i]} ${a[i-1]} ${a[ 1 + 1 ]}"
echo "c20: ${a}"
echo "c21: ${#a[1]}"
printf 'c22: [%s]\n' "${!a[@]}"
printf 'c23: [%s]\n' "${a[@]^}"
printf 'c24: [%s]\n' "x${a[@]}y"
b=("${a[@]}")
echo "c25: ${#b[@]}"
c=(${a[@]})
echo "c26: ${#c[@]}"
d=("${a[*]}")
echo "c27: ${#d[@]}"
echo "c28: ${s[@]: -1} ${s[@]: -2:1}"
for idx in "${!s[@]}"; do echo "c29: s[$idx]=${s[idx]}"; done
echo "c30: ${#s[3]} ${#s[@]}"
echo "c31: ${a[@]:0:2}+${a[@]:4}"
