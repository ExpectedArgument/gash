# test / [ builtin: argument-count forms, -a/-o, string vs numeric
# t runs a command and appends its status to $r; line prints and resets $r.
r=""
t() { "$@"; r+=" $?"; }
line() { echo "$1:$r"; r=""; }
t test; t [ ]; line c1
t test x; t test ''; t [ -n ]; t [ -z ]; t [ ! ]; line c2
t [ ! x ]; t [ ! '' ]; t [ -n '' ]; t [ -z '' ]; t [ -n x ]; line c3
t [ a = a ]; t [ a = b ]; t [ a != b ]; t [ a == a ]; line c4
t [ 2 -eq 2 ]; t [ 2 -ne 2 ]; t [ 1 -lt 2 ]; t [ 2 -le 1 ]; t [ 3 -gt 2 ]; t [ 3 -ge 4 ]; line c5
t [ ! a = b ]; t [ ! -z x ]; t [ '(' x ')' ]; line c6
t [ a = a -a b = b ]; t [ a = a -a b = c ]; t [ a = b -o b = b ]; t [ a = b -o b = c ]; line c7
t [ -n x -a -z '' ]; t [ ! a = a -o b = b ]; line c8
t [ a '<' b ]; t [ b '<' a ]; t [ b '>' a ]; line c9
t [ abc = 'a*' ]; t [ 010 -eq 10 ]; t [ ' 5' -eq 5 ]; line c10
t [ x -eq 1 ] 2>/dev/null; t [ 1 -eq ] 2>/dev/null; t [ a 2>/dev/null; line c11
t [ -eq ]; t [ = ]; t [ -a ]; t [ -o ]; line c12
t [ '(' -n x ')' ]; t [ ! '(' x ')' ]; line c13
t [ '(' a = a ')' -a '(' b = b ')' ]; line c14
t test -n ""; t test 5 -gt 10; t test abc != abc; line c15
t [ -1 -lt 0 ]; t [ +3 -eq 3 ]; line c16
t [ "" = "" ]; t [ -z "$unset" ]; t [ -n "$unset" ]; line c17
t [ -n $unset ]; line c18
t [ 1 -eq 1 -a 2 -eq 3 -o 4 -eq 4 ]; line c19
myv=1
t [ -v myv ]; t [ -v nope_var ]; line c20
x=5
t [ $x -eq 5 ]; t test "$x" -gt 4; line c21
t [ -o noglob ]; t [ -t 0 ]; line c22
t [ x = x = x ] 2>/dev/null; t [ a b c ] 2>/dev/null; line c23
t [ ! ! x ]; t [ ! ! '' ]; line c24
t [ 0 -eq -0 ]; t [ 2147483648 -gt 2147483647 ]; line c25
t [ 1.5 -eq 1 ] 2>/dev/null; line c26
t [ abc \> abd ]; t [ abd \> abc ]; line c27
t [ -z ' ' ]; t [ ' ' ]; line c28
t [ '!' = '!' ]; t [ '(' = '(' ]; t [ -n = -n ]; line c29
t [ ! = x ] 2>/dev/null; line c30
t [ -e ]; t [ -f ]; t [ -d ]; line c31
t test a = a -a ''; t test a = a -o ''; line c32
if [ "$x" -gt 3 ] && [ "$x" -lt 10 ]; then echo "c33: combined with &&"; fi
[ "$x" = 5 ] || echo never; echo "c34: string compare of number $?"
v=""
if [ "x$v" = "x" ]; then echo "c35: x-prefix idiom"; fi
[ -n "$v" ] && echo never || echo "c36: empty var"
t [ 00 -eq 0 ]; t [ 09 -eq 9 ]; line c37
t [ ]; t [ '' ]; t [ "" "" ] 2>/dev/null; line c38
t test ! -n ""; t test -z -n; line c39
t [ 3 -lt 20 ]; t [ 3 '<' 20 ]; line c40
