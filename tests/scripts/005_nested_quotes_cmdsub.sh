# nested quoting inside $(...)
x="outer value"
echo "c1: $(echo "inner $x")"
echo "c2: $(echo "a  b" | tr -s ' ')"
echo "c3: $(echo 'single "dq" inside')"
echo "c4: $(echo "$(echo "deep $(echo "deeper")")")"
y="$(printf '%s' "it's")"
echo "c5: $y"
echo "c6: $(echo "\$x")"
echo "c7: $(echo '$x')"
echo "c8: $(echo "a)b")"
echo "c9: $(echo "case )")"
echo "c10: $( echo "spaces  kept" )"
z=$(echo "  lead and trail  ")
echo "c11: [$z]"
echo c12: [$(echo "  lead and trail  ")]
echo "c13: `echo "backtick \"dq\""`"
echo "c14: $(echo "a\"b")"
arr=("$(echo "one two")" "$(echo three)")
echo "c15: ${#arr[@]} [${arr[0]}]"
echo "c16: ${x:+"$(echo "alt $x")"}"
echo "c17: $(echo "${x// /_}")"
echo "c18: $(cat <<EOF
heredoc in cmdsub $x
EOF
)"
echo "c19: $( (echo subshell inside) )"
echo "c20: $(case a in a) echo case-in-cmdsub;; esac)"
echo "c21: $(echo hi # comment with ) paren
)"
echo "c22: $(echo "{a,b}" {c,d})"
echo "c23: $(echo '$(not run)')"
q="$(echo "x  y")"; set -- $q; echo "c24: $#"
set -- "$(echo "x  y")"; echo "c25: $#"
echo "c26: $(printf '%s|' "$(echo a b)" "$(echo 'c d')")"
echo "c27: $(echo "${x%% *}") $(echo "${x#* }")"
echo "c28: $(x="shadow"; echo "$x") $x"
echo "c29: $(echo \"escaped\")"
echo c30: "$(echo 'a'"b"'c')"
echo "c31: $(echo $(echo '"q"'))"
v=$(echo "line1
line2")
echo "c32: ${v//$'\n'/+}"
echo "c33: $(if [ "a b" = "a b" ]; then echo "eq"; fi)"
echo "c34: $(echo "a'b")"
