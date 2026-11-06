# mapfile / readarray
mapfile -t arr <<< $'one\ntwo\nthree'
echo "c1: ${#arr[@]} [${arr[0]}] [${arr[2]}]"
mapfile arr2 <<< $'x\ny'
printf 'c2: [%s]' "${arr2[@]}"; echo
readarray -t arr3 < <(printf 'a\nb\nc\nd\n')
echo "c3: ${arr3[*]}"
mapfile -t -n 2 arr4 < <(printf '1\n2\n3\n')
echo "c4: ${arr4[*]}"
mapfile -t -s 1 arr5 < <(printf '1\n2\n3\n')
echo "c5: ${arr5[*]}"
mapfile -t -d , arr6 < <(printf 'a,b,c')
echo "c6: ${#arr6[@]} ${arr6[*]}"
mapfile -t arr7 < /dev/null
echo "c7: ${#arr7[@]}"
mapfile -t <<< $'r1\nr2'
echo "c8: ${MAPFILE[*]}"
arr8=(old values here)
mapfile -t arr8 <<< "new"
echo "c9: ${#arr8[@]} ${arr8[*]}"
mapfile -t -O 2 arr9 <<< $'p\nq'
echo "c10: ${!arr9[*]} ${arr9[*]}"
printf 'l1\n\nl3\n' | { mapfile -t a; echo "c11: ${#a[@]} [${a[1]}]"; }
mapfile -t arr10 <<< ""
echo "c12: ${#arr10[@]} [${arr10[0]}]"
mapfile -t arr11 < <(printf 'no trailing newline')
echo "c13: ${#arr11[@]} [${arr11[0]}]"
tmp=$(mktemp -d) || exit 1
printf 'f1\nf2\nf3\n' > "$tmp/data"
mapfile -t fromfile < "$tmp/data"
rm -rf "$tmp"
printf 'c14: %s\n' "${fromfile[@]}"
mapfile -t crlf < <(printf 'a\r\nb\r\n')
echo "c15: ${#crlf[0]} $(printf '%s' "${crlf[0]}" | cat -v)"
mapfile -t -s 1 -n 2 both < <(printf '%s\n' a b c d e)
echo "c16: ${both[*]}"
mapfile -t spaces <<< $'  lead\ntrail  \n in ner '
printf 'c17: [%s]\n' "${spaces[@]}"
readarray -t sorted < <(printf '%s\n' c a b | sort)
echo "c18: ${sorted[*]}"
mapfile -t -d '' nul < <(printf 'x\0y\0')
echo "c19: ${#nul[@]} ${nul[1]}"
mapfile -t lines < <(printf 'k1=v1\nk2=v2\n')
for l in "${lines[@]}"; do echo "c20: key=${l%%=*} val=${l#*=}"; done
mapfile -t -n 0 all < <(printf '1\n2\n')
echo "c21: ${#all[@]}"
out=$(printf 'm1\nm2\n' | { mapfile -t m; echo "${#m[@]}"; }); echo "c22: $out"
