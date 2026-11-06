# arithmetic assignment operators, increments, (( )) exit status
x=10
(( x += 5 )); echo "c1: $x"
(( x -= 3 )); echo "c2: $x"
(( x *= 2 )); echo "c3: $x"
(( x /= 5 )); echo "c4: $x"
(( x %= 3 )); echo "c5: $x"
(( x <<= 3 )); echo "c6: $x"
(( x >>= 1 )); echo "c7: $x"
(( x |= 1 )); echo "c8: $x"
(( x &= 6 )); echo "c9: $x"
(( x ^= 15 )); echo "c10: $x"
i=5
echo "c11: $((i++)) $i"
echo "c12: $((++i)) $i"
echo "c13: $((i--)) $i"
echo "c14: $((--i)) $i"
j=0
(( j++ )); echo "c15: status $? j=$j"
(( j++ )); echo "c16: status $? j=$j"
(( 0 )); echo "c17: $?"
(( 1 )); echo "c18: $?"
(( -1 )); echo "c19: $?"
(( a = 3, b = a * 2 )); echo "c20: $a $b"
(( c = d = 7 )); echo "c21: $c $d"
echo "c22: $(( n = 4 )) $n"
k=1
echo "c23: $(( k++ + k++ )) $k"
echo "c24: $(( k += k )) $k"
arr=(1 2 3)
(( arr[1] += 10 )); echo "c25: ${arr[*]}"
(( arr[3] = arr[0] + arr[2] )); echo "c26: ${arr[*]}"
idx=2
(( arr[idx]++ )); echo "c27: ${arr[*]}"
declare -A h
h[k]=5
(( h[k] *= 3 )); echo "c28: ${h[k]}"
if (( 3 > 2 )); then echo "c29: if arith"; fi
while (( i < 8 )); do (( i++ )); done; echo "c30: $i"
(( m = 5 > 3 ? 100 : 200 )); echo "c31: $m"
s=abc
(( s = 3 )); echo "c32: $s"
unset q; (( q++ )); echo "c33: $q"
unset q2; (( ++q2 )); echo "c34: $q2"
v=7; (( v = v % 4 )); echo "c35: $v"
(( 5 > 3 )) && echo "c36: and-list"
(( 0 )) || echo "c37: or-list"
w="2+3"; (( w *= 2 )); echo "c38: $w"
(( t = 0 )); echo "c39: status $? t=$t"
(( t = 1 )); echo "c40: status $? t=$t"
cnt=0; for f in a b c; do (( cnt++ )); done; echo "c41: $cnt"
((y=2)); ((y**=3)) 2>/dev/null; echo "c42: $y"
z=10; (( z -= z / 3 )); echo "c43: $z"
((   spaced   =   1   +   1   )); echo "c44: $spaced"
