# if / elif / else, and-or lists
check() {
  if [ "$1" -lt 0 ]; then echo "neg"
  elif [ "$1" -eq 0 ]; then echo "zero"
  elif [ "$1" -lt 10 ]; then echo "small"
  else echo "big"
  fi
}
for v in -5 0 3 42; do echo "c1: $v $(check $v)"; done
if true; then echo "c2: true branch"; fi
if false; then echo "c3: no"; else echo "c3: else branch"; fi
if ! false; then echo "c4: negated"; fi
if true && false; then echo c5: no; else echo "c5: and-false"; fi
if false || true; then echo "c6: or-true"; fi
if { echo "c7: in condition group"; false; }; then echo c7 no; else echo "c7: group false"; fi
if (exit 0); then echo "c8: subshell condition"; fi
x=$(if true; then echo "c9: if in cmdsub"; fi); echo "$x"
if false; then :; fi; echo "c10: status after if with no branch $?"
if false; then :; else (exit 4); fi; echo "c11: status $?"
if true; then (exit 3); fi; echo "c12: status $?"
if [ -n "x" ]; then
  if [ -z "" ]; then
    echo "c13: nested"
  fi
fi
if true; then echo "c14: one-liner"; elif false; then :; else :; fi
if
  false
then
  echo no
elif
  true
then
  echo "c15: multiline elif"
fi
true && echo "c16: and-list" || echo "no"
false && echo "no" || echo "c17: or fallback"
false || false || echo "c18: chained or"
true && true && echo "c19: chained and"
if test 2 -gt 1; then echo "c20: test cmd"; fi
r=0; if [[ $r == 0 ]]; then echo "c21: [[ in if"; fi
if false; then echo no; elif false; then echo no; fi; echo "c22: no branch taken $?"
if false; then echo no; elif (exit 6); then echo no; else echo "c23: else after failing elif $?"; fi
if true; then false; fi; echo "c24: body status $?"
true || echo never; echo "c25: $?"
false && echo never; echo "c26: $?"
if true
then echo "c27: then on next line"
fi
if [ 1 = 1 ] ; then : ; fi ; echo "c28: semicolons $?"
if true; then
  :
elif false; then
  :
else
  :
fi
echo "c29: empty bodies ok"
ok=1
[ "$ok" = 1 ] && { echo "c30: brace group"; echo "c30: second"; }
! [ 1 = 2 ] && echo "c31: negated test"
if if true; then false; fi; then echo no; else echo "c32: nested if as condition"; fi
