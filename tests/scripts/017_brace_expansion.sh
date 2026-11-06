# brace expansion
echo "c1:" {a,b}{1,2}
echo "c2:" {1..10..3}
echo "c3:" {a..e}
echo "c4:" {05..10}
echo "c5:" {e..a}
echo "c6:" {10..1..4}
echo "c7:" x{a,{b,c}d,e}y
echo "c8:" pre-{x,y}-post
echo "c9:" "{a,b}" '{a,b}'
echo "c10:" {a}
echo "c11:" {}
echo "c12:" a{,b,c}
echo "c13:" {-2..2}
echo "c14:" {a..e..2}
n=3
echo "c15:" {1..$n}
echo "c16:" {1..3}{x,y}
echo "c17:" \{a,b\}
echo "c18:" {a,b\,c}
echo "c19:" {"a b",c}
x=val
echo "c20:" {$x,other}
echo "c21:" {1..1}
echo "c22:" {001..3}
echo "c23:" {Y..b}
echo "c24:" file{1..3}.txt
echo "c25:" {a,b}{,}
arr=({1..3})
echo "c26: ${#arr[@]}"
echo "c27:" {a..c}{1..2}
echo "c28:" a{b{c,d}e,f}g
echo "c29:" ${x}{1,2}
echo "c30:" {1,2}"quoted suffix"
echo "c31:" {-01..02}
echo "c32:" {a,b}/{c,d}
echo "c33:" {1..10..-3}
echo "c34:" {3..1}
echo "c35:" {a..c..-1}
echo "c36:" {1..2}{a..b}{X,Y}
echo "c37:" {,a,b}
echo "c38:" a{1..3
echo "c39:" a{1,2
echo "c40:" {1.5..3}
echo "c41:" {a,b}"{c,d}"
echo "c42:" $'{a,b}'
y="{p,q}"
echo "c43:" $y
for i in {1..3}; do echo -n "c44:$i "; done; echo
echo "c45:" {0..20..5}
echo "c46:" {x..z}{0..1}
echo "c47:" @{a,b}@
echo "c48:" {10..8}
echo "c49:" {aa,bb}{cc,}
