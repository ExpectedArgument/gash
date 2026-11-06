# case statement: patterns, |, ;&, ;;&
classify() {
  case $1 in
    [0-9]) echo digit;;
    [a-z]|[A-Z]) echo letter;;
    *.txt) echo text-file;;
    ''|' ') echo blank;;
    "quoted*") echo literal-star;;
    *) echo other;;
  esac
}
for v in 5 x Q notes.txt '' ' ' 'quoted*' 'quotedX' '??'; do echo "c1: [$v] $(classify "$v")"; done
case abc in a*) echo "c2: a*";;& *c) echo "c2: *c";;& b*) echo "c2: no";; *) echo "c2: final default";; esac
case 1 in 1) echo "c3: one";& 2) echo "c3: two (fell)";& 3) echo "c3: three (fell)";; 4) echo "c3: no";; esac
case x in y) echo no;; esac; echo "c4: no match status $?"
case x in x) (exit 3);; esac; echo "c5: status $?"
case "a b" in "a b") echo "c6: quoted with space";; esac
pat="a*"
case abc in $pat) echo "c7: unquoted var pattern globs";; esac
case abc in "$pat") echo no;; *) echo "c8: quoted var pattern literal";; esac
case 'a*' in "$pat") echo "c9: literal match";; esac
case foo in
  (foo) echo "c10: leading paren";;
esac
case X in [[:upper:]]) echo "c11: class";; esac
case - in [a-]) echo "c12: dash in bracket";; esac
case ']' in []]) echo "c13: bracket first";; esac
case '!' in [!a]) echo "c14: negated set";; esac
case $undefined in '') echo "c15: empty matches ''";; esac
case "x" in
  x) echo "c16: no trailing ;;"
esac
case a in a) ;; esac; echo "c17: empty body status $?"
x=hello
case $x in h*o) echo "c18: h*o";; esac
case $x in *ll*) echo "c19: contains";; esac
case 10 in [0-9]) echo no;; [0-9][0-9]) echo "c20: two digits";; esac
r=$(case y in y|Y|yes) echo "c21: yes";; esac); echo "$r"
case "a.b" in *.*) echo "c22: has dot";; esac
case abc in ?b?) echo "c23: ?b?";; esac
case "back\\slash" in *\\*) echo "c24: backslash";; esac
case 2 in 1) echo one;;& 2) echo "c25: two";;& [0-9]) echo "c25: digit";;& *) echo "c25: any";; esac
case z in a) echo a;& z) echo "c26: z";& y) echo "c26: y fell";; esac
case word in w*|x*) echo "c27: first alternative";; esac
case "$x" in
  hel*)
    echo "c28: multi-line body 1"
    echo "c28: multi-line body 2"
    ;;
  *)
    echo no
    ;;
esac
for f in a.sh b.py c.rs d; do
  case $f in
    *.sh) lang=bash;;
    *.py) lang=python;;
    *.rs) lang=rust;;
    *) lang=unknown;;
  esac
  echo "c29: $f -> $lang"
done
case $((1+1)) in 2) echo "c30: arithmetic word";; esac
case "$(echo hi)" in hi) echo "c31: cmdsub word";; esac
v=in
case $v in in) echo "c32: keyword-like word";; esac
case "*" in \*) echo "c33: escaped star pattern";; esac
case abc in *) echo "c34: first";; abc) echo "c34: never";; esac
y="b"
case b in a|$y|c) echo "c35: var in alternative";; esac
case esac in "esac") echo "c36: esac as quoted pattern";; esac
