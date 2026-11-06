# [[ =~ ]] and BASH_REMATCH
s="key=value123"
if [[ $s =~ ^([a-z]+)=([a-z]+)([0-9]+)$ ]]; then
  echo "c1: ${BASH_REMATCH[0]} | ${BASH_REMATCH[1]} | ${BASH_REMATCH[2]} | ${BASH_REMATCH[3]}"
  echo "c2: ${#BASH_REMATCH[@]}"
fi
[[ abc =~ x ]]; echo "c3: $? count=${#BASH_REMATCH[@]}"
re='^[0-9]{3}-[0-9]{4}$'
[[ 555-1234 =~ $re ]]; echo "c4: $?"
[[ 555-12345 =~ $re ]]; echo "c5: $?"
[[ a.c =~ a.c ]]; echo "c6: $?"
[[ abc =~ "a.c" ]]; echo "c7: quoted literal $?"
[[ a.c =~ "a.c" ]]; echo "c8: $?"
[[ abc =~ a"."c ]]; echo "c9: partial quote $?"
[[ "hello world" =~ o\ w ]]; echo "c10: escaped space $?"
[[ "hello world" =~ (o)(.)(w) ]] && echo "c11: [${BASH_REMATCH[2]}]"
[[ xyz =~ y ]] && echo "c12: unanchored [${BASH_REMATCH[0]}]"
[[ aaa =~ a* ]] && echo "c13: greedy [${BASH_REMATCH[0]}]"
[[ abcabc =~ (abc)+ ]] && echo "c14: [${BASH_REMATCH[0]}] [${BASH_REMATCH[1]}]"
[[ ab =~ (x)?(a)(b) ]] && echo "c15: [${BASH_REMATCH[1]}] [${BASH_REMATCH[2]}] ${#BASH_REMATCH[@]}"
bad_re='('
[[ a =~ $bad_re ]] 2>/dev/null; echo "c16: invalid regex status $?"
date="2024-01-15"
if [[ $date =~ ^([0-9]{4})-([0-9]{2})-([0-9]{2})$ ]]; then
  y=${BASH_REMATCH[1]} m=${BASH_REMATCH[2]} d=${BASH_REMATCH[3]}
  echo "c17: $d/$m/$y"
fi
[[ "FOO" =~ ^[[:upper:]]+$ ]]; echo "c18: $?"
[[ "a|b" =~ a\|b ]]; echo "c19: $?"
[[ "ab" =~ a|x ]]; echo "c20: alternation $?"
email="user@example.com"
[[ $email =~ ^[^@]+@[^@]+\.[a-z]+$ ]] && echo "c21: email ok"
v="x1y22z333"
while [[ $v =~ [0-9]+ ]]; do echo "c22: ${BASH_REMATCH[0]}"; v=${v#*"${BASH_REMATCH[0]}"}; done
[[ abc =~ ^ ]]; echo "c23: empty match $? [${BASH_REMATCH[0]}]"
[[ "a b" =~ ^a\ b$ ]]; echo "c24: $?"
[[ 'a$b' =~ a\$b ]]; echo "c25: $?"
shopt -s nocasematch
[[ ABC =~ ^abc$ ]]; echo "c26: nocasematch regex $?"
shopt -u nocasematch
[[ ABC =~ ^abc$ ]]; echo "c27: $?"
re2='(a+)(b*)'
[[ xaabbby =~ $re2 ]] && echo "c28: ${BASH_REMATCH[*]}"
[[ "x.y" =~ ^x\.y$ ]]; echo "c29: $?"
[[ "xzy" =~ ^x\.y$ ]]; echo "c30: $?"
re3='^x\.y$'
[[ "x.y" =~ $re3 ]]; echo "c31: $?"
[[ "foo bar" =~ ^(foo)\ (bar)$ ]] && echo "c32: ${BASH_REMATCH[2]}-${BASH_REMATCH[1]}"
[[ "line" =~ ^l(i)?n(x)?e$ ]] && echo "c33: [${BASH_REMATCH[1]}] [${BASH_REMATCH[2]}] ${#BASH_REMATCH[@]}"
[[ "ab12" =~ ^[a-z]+([0-9])+$ ]] && echo "c34: last repetition [${BASH_REMATCH[1]}]"
[[ "tab	sep" =~ [[:blank:]] ]]; echo "c35: $?"
[[ "abc" =~ ^.{2}$ ]]; echo "c36: $?"
[[ "abc" =~ ^.{2,}$ ]]; echo "c37: $?"
[[ "aXb" =~ a[^a-z]b ]]; echo "c38: $?"
[[ "a]b" =~ a[]]b ]]; echo "c39: $?"
[[ "" =~ ^$ ]]; echo "c40: $?"
if [[ ! abc =~ ^z ]]; then echo "c41: negated regex"; fi
ver="v1.2.3-rc1"
[[ $ver =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)(-(.*))?$ ]] && echo "c42: ${BASH_REMATCH[1]} ${BASH_REMATCH[2]} ${BASH_REMATCH[3]} [${BASH_REMATCH[5]}]"
