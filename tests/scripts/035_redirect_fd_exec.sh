# file descriptors and exec redirections
tmp=$(mktemp -d) || exit 1
cd "$tmp" || exit 1
exec 3>out3.txt
echo "line1" >&3
echo "line2" >&3
exec 3>&-
echo "c1: $(cat out3.txt | tr '\n' ' ')"
printf 'r1\nr2\nr3\n' > in.txt
exec 4<in.txt
read -r a <&4
read -r b <&4
read -r -u 4 c
exec 4<&-
echo "c2: $a $b $c"
exec 5>>out3.txt
echo appended >&5
exec 5>&-
echo "c3: $(tail -n 1 out3.txt)"
( echo x >&3 ) 2>/dev/null; echo "c4: closed fd write status $?"
exec 6>&1
x=$( echo to-captured; echo to-orig >&6 )
echo "c5: [$x]"
exec 6>&-
{ echo a; echo b >&2; } 2>&1 | sed 's/^/c6: /'
exec 7<>rw.txt
echo "written" >&7
exec 7>&-
echo "c7: $(cat rw.txt)"
echo "c8: $( { echo via3 >&3; } 3>&1 )"
f() { echo "c9: fd from func" >&4; }
f 4>&1
exec {fd}>auto.txt
echo "auto fd" >&$fd
exec {fd}>&-
echo "c10: $(cat auto.txt) fd>=10:$(( fd >= 10 ))"
echo "c11: $( (echo inner-err >&2) 3>&2 2>&1 )"
exec 8>&1
exec >redirected.txt
echo "goes to file"
exec 1>&8 8>&-
echo "c12: $(cat redirected.txt)"
exec 3<in.txt
while read -r line <&3; do echo "c13: $line"; done
exec 3<&-
exec 3<in.txt 4<in.txt
read -r x1 <&3; read -r x2 <&3; read -r y1 <&4
exec 3<&- 4<&-
echo "c14: $x1 $x2 $y1"
{ echo "c15: grouped to fd"; } 9>&1 >&9
exec 5>dup.txt
echo one >&5
exec 6>&5
echo two >&6
exec 5>&- 6>&-
echo "c16: $(tr '\n' ' ' < dup.txt)"
exec 3>&1
out=$( { echo "c17: to stdout of group"; echo "c17: bypass" >&3; } | sed 's/^/piped /' )
exec 3>&-
echo "$out"
cat <&- 2>/dev/null; echo "c18: closed stdin cat status $?"
x=$(exec 2>&1; echo "c19: exec in subshell" >&2); echo "$x"
exec 4>moved.txt
echo "moved" 1>&4-
echo "c20: $(cat moved.txt)"
( echo "x" >&4 ) 2>/dev/null; echo "c21: fd moved and closed status $?"
cd .. && rm -rf "$tmp"
