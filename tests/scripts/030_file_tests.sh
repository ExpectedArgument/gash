# file test operators on files created by the script
tmp=$(mktemp -d) || exit 1
cd "$tmp" || exit 1
echo content > file.txt
: > empty.txt
mkdir adir
# t runs a test and appends 0/1 to $r; line prints and resets $r.
r=""
t() { if "$@"; then r+=" 0"; else r+=" 1"; fi; }
line() { echo "$1:$r"; r=""; }
t [ -f file.txt ]; t [ -f adir ]; t [ -f missing ]; line "c1: -f"
t [ -d adir ]; t [ -d file.txt ]; t [ -d missing ]; line "c2: -d"
t [ -e file.txt ]; t [ -e adir ]; t [ -e missing ]; line "c3: -e"
t [ -s file.txt ]; t [ -s empty.txt ]; t [ -s missing ]; line "c4: -s"
t [ -r file.txt ]; t [ -r missing ]; line "c5: -r"
t [ -w file.txt ]; t [ -w missing ]; line "c6: -w"
touch -d '2020-01-01 00:00:00' old.txt
touch -d '2021-06-01 00:00:00' new.txt
t [ new.txt -nt old.txt ]; t [ old.txt -nt new.txt ]; t [ new.txt -nt missing ]; t [ missing -nt new.txt ]; line "c7: -nt"
t [ old.txt -ot new.txt ]; t [ new.txt -ot old.txt ]; t [ missing -ot old.txt ]; line "c8: -ot"
[[ -f file.txt && -d adir ]]; echo "c9: $?"
[[ -e missing || -s file.txt ]]; echo "c10: $?"
[[ ! -e missing ]]; echo "c11: $?"
[[ new.txt -nt old.txt ]]; echo "c12: $?"
[[ file.txt -ef file.txt ]]; echo "c13: -ef $?"
[[ file.txt -ef empty.txt ]]; echo "c14: $?"
f=file.txt
[ -f "$f" ] && echo "c15: var path"
[ -f ./adir/../file.txt ]; echo "c16: relative dots $?"
t [ -L file.txt ]; t [ -h file.txt ]; line "c17: -L/-h"
mkdir -p a/b/c
[ -d a/b/c ] && echo "c18: mkdir -p"
cd a/b && [ -d c ] && echo "c19: cd into nested"; cd ../..
[ -e "file with space.txt" ]; echo "c20: $?"
: > "file with space.txt"
[ -e "file with space.txt" ]; echo "c21: $?"
for x in *.txt; do [ -s "$x" ] && echo "c22: nonempty $x"; done
[ -e "" ]; echo "c23: empty name $?"
[ -d . ] && [ -d .. ] && echo "c24: dot dirs"
echo more >> empty.txt
[ -s empty.txt ] && echo "c25: no longer empty"
[[ -f file.txt ]] && [[ ! -d file.txt ]] && echo "c26: file not dir"
test -e adir/; echo "c27: trailing slash dir $?"
test -e file.txt/; echo "c28: trailing slash file $?"
[ -f adir/nope ] || echo "c29: missing nested"
read -r content < file.txt
[[ -s file.txt && $content == content ]] && echo "c30: content check"
t test -f file.txt -a -d adir; t test -f missing -o -d adir; t test ! -e missing; line "c31: combined"
[ -d sub ] || mkdir sub; [ -d sub ] && echo "c32: created on demand"
rmdir sub; [ -e sub ]; echo "c33: removed $?"
cd .. && rm -rf "$tmp"
