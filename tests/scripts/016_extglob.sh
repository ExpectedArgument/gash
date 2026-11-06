# extended globbing: @() !() +() *() ?()
shopt -s extglob
tmp=$(mktemp -d) || exit 1
cd "$tmp" || exit 1
: > a.c; : > b.c; : > a.h; : > main.c; : > test.o; : > aa; : > aaa; : > ab
echo "c1:" @(a|b).c
echo "c2:" !(*.c)
echo "c3:" +(a)
echo "c4:" *(a)b
echo "c5:" ?(a)b
echo "c6:" !(a*)
echo "c7:" *.@(c|h)
echo "c8:" @(nomatch|zzz)
for w in foo foobar barfoo bar ""; do
  case $w in
    @(foo|bar)) echo "c9: $w exact";;
    *(foo)bar) echo "c9: $w foo*bar";;
    +(?)) echo "c9: $w other";;
    *) echo "c9: [$w] empty";;
  esac
done
[[ abc123 == +([a-z])+([0-9]) ]] && echo "c10: letters then digits"
[[ abc == !(abc) ]] || echo "c11: negation fails on exact"
[[ abd == !(abc) ]] && echo "c12: negation matches other"
x="  trim me  "
y="${x##+([[:space:]])}"; y="${y%%+([[:space:]])}"
echo "c13: [$y]"
v="aaa-bbb"
echo "c14: ${v//+(a)/X}"
[[ "x.tar.gz" == *.@(tar.gz|tgz) ]] && echo "c15: archive"
[[ file.txt == !(*.txt) ]] || echo "c16: excluded txt"
echo "c17:" @(a|aa|aaa)
echo "c18:" a@(.c|.h)
echo "c19:" !(a|aa|aaa|ab)
[[ "" == ?(x) ]] && echo "c20: ?() matches empty"
[[ xx == ?(x) ]] || echo "c21: ?() at most one"
[[ xyxy == *(xy) ]] && echo "c22: *() repeats"
[[ "" == +(x) ]] || echo "c23: +() needs one"
[[ a.b.c == +([a-z]).+([a-z]).+([a-z]) ]] && echo "c24: nested pattern"
[[ foo == @(f*|g*) ]] && echo "c25: glob inside alternation"
[[ abc == a!(b)c ]] || echo "c26: a!(b)c vs abc"
[[ axc == a!(b)c ]] && echo "c27: a!(b)c vs axc"
num="00042"
echo "c28: ${num##+(0)}"
p='@(x|y)'
[[ y == $p ]] && echo "c29: extglob from variable"
cd .. && rm -rf "$tmp"
