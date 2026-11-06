# pathname expansion in a temp dir
tmp=$(mktemp -d) || exit 1
cd "$tmp" || exit 1
: > apple.txt; : > banana.txt; : > cherry.log; : > a1; : > a2; : > b1; : > .hidden; mkdir dir1 dir2
echo "c1:" *
echo "c2:" *.txt
echo "c3:" a?
echo "c4:" [ab]*
echo "c5:" [a-b]?
echo "c6:" [!a]*
echo "c7:" [^a]*
echo "c8:" *.nomatch
echo "c9:" "*.txt"
echo "c10:" d*/
shopt -s nullglob
echo "c11:" *.nomatch "|"
set -- *.nomatch; echo "c12: $#"
shopt -u nullglob
shopt -s dotglob
echo "c13:" *
shopt -u dotglob
echo "c14:" .h*
echo "c15:" .*
p='*.log'
echo "c16:" $p
echo "c17:" "$p"
shopt -s failglob
( echo *.nomatch; echo "c18: not reached" ) 2>/dev/null; echo "c18: status $?"
shopt -u failglob
mkdir -p sub/deep; : > sub/x.txt; : > sub/deep/y.txt
echo "c19:" */*.txt
shopt -s globstar
echo "c20:" **/*.txt
echo "c21:" sub/**
shopt -u globstar
echo "c22:" [[:digit:]]*
echo "c23:" *[[:digit:]]
echo "c24:" ?????.txt
echo "c25:" \*
files=(*.txt)
echo "c26: ${#files[@]} ${files[1]}"
for f in a*; do echo "c27: $f"; done
case apple.txt in *.txt) echo "c28: case glob";; esac
[[ banana.txt == b*.txt ]] && echo "c29: [[ glob"
echo "c30:" [b-c]*.*
set -f
echo "c31:" *
set +f
GLOBIGNORE='*.txt'
echo "c32:" *
unset GLOBIGNORE
echo "c33:" "a"*
echo "c34:" 'a'?
x=a
echo "c35:" ${x}*
echo "c36:" sub/*
echo "c37:" */
echo "c38:" sub/deep/*.txt
echo "c39:" *[0-9]
echo "c40:" [!ab.]*
: > 'sp ace.txt'
for f in sp*; do echo "c41: [$f]"; done
arr=(*.nomatch)
echo "c42: ${#arr[@]} [${arr[0]}]"
echo "c43:" *.{txt,log}
echo "c44:" *a*a*
cd .. && rm -rf "$tmp"
