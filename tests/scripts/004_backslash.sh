# backslash escaping in various contexts
echo c1: a\ b
echo c2: a\\b
echo "c3: a\\b"
echo 'c4: a\\b'
echo c5: \$HOME_not
echo c6: \'single\'
echo "c7: \n stays in dq"
echo c8: \n becomes n
echo "c9: line\
continued"
echo c10: \*\?\[
x='a\b'
echo "c11: $x"
echo c12: $x
printf '%s\n' "c13: \\\\"
printf '%s\n' c14: \\\\
echo "c15: \\\$"
echo c16: \\\$
y="back\\slash"
echo "c17: $y ${#y}"
echo 'c18: '\\'"'
case 'a*b' in a\*b) echo "c19: escaped star matches literally";; *) echo "c19: no";; esac
case 'axb' in a\*b) echo "c20: matched";; *) echo "c20: escaped star does not glob";; esac
echo c21: "\"nested\" \"quotes\""
z=\a\b\c
echo "c22: $z"
echo "c23: \{ \} \( \)"
echo c24: \{a,b\}
echo c25: \;\&\|\<\>
w="tab\there"
echo "c26: $w"
echo c27: \
continued unquoted
v="a\
b"
echo "c28: $v"
echo "c29: \\\\ is two"
echo c30: \\\\ unquoted
s='\\'
echo "c31: ${#s}"
echo c32: "\a\b\c"
echo "c33: a\"b'c"
p='x\*y'
case 'x*y' in $p) echo "c34: escaped glob in var pattern";; *) echo "c34: no";; esac
