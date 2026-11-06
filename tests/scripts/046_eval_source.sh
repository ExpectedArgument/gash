# eval and source / .
x=10
eval 'echo "c1: $x"'
cmd='y=$((x * 2))'
eval "$cmd"; echo "c2: $y"
name=dynvar
eval "$name=hello"
echo "c3: $dynvar"
eval "echo c4: \$$name"
eval echo "c5: '\$x literal'"
eval 'for i in 1 2; do echo "c6: $i"; done'
eval "false"; echo "c7: $?"
eval "(exit 6)"; echo "c8: $?"
eval ""; echo "c9: empty eval $?"
( eval 'if then' ) 2>/dev/null; echo "c10: syntax error status $?"
arr=(a "b c")
eval "copy=($(printf '%q ' "${arr[@]}"))"
echo "c11: ${#copy[@]} [${copy[1]}]"
q="it's"
eval "r=$(printf '%q' "$q")"; echo "c12: $r"
eval "f() { echo 'c13: defined via eval'; }"; f
a=1; eval a=2 b=3; echo "c14: $a $b"
set -- p1 p2
eval 'echo "c15: $1 $2"'
eval echo '"c16: $(( 3 + 4 ))"'
tmp=$(mktemp -d) || exit 1
cat > "$tmp/lib.sh" <<'EOF'
lib_var="from lib"
lib_func() { echo "lib_func called with $#: $*"; }
echo "c17: sourcing: args=$# first=${1-none}"
EOF
source "$tmp/lib.sh"
echo "c18: $lib_var"
echo "c19: $(lib_func a b)"
. "$tmp/lib.sh" s1 s2
echo "c20: after dot with args, outer \$1=$1"
cat > "$tmp/ret.sh" <<'EOF'
echo "c21: before return"
return 4
echo "never"
EOF
source "$tmp/ret.sh"; echo "c21: status $?"
cat > "$tmp/vars.sh" <<'EOF'
counter=$((counter + 1))
EOF
counter=0; . "$tmp/vars.sh"; . "$tmp/vars.sh"; echo "c22: $counter"
( source "$tmp/nonexistent.sh" ) 2>/dev/null; echo "c23: missing file status $?"
fsrc() { local inner=local_in_func; source "$tmp/vars.sh"; echo "c24: $counter"; }
fsrc
cat > "$tmp/setpos.sh" <<'EOF'
set -- changed
EOF
set -- orig; source "$tmp/setpos.sh"; echo "c25: $1"
set -- orig; source "$tmp/setpos.sh" arg; echo "c26: $1"
cat > "$tmp/localvar.sh" <<'EOF'
local in_sourced=yes
EOF
lf() { source "$tmp/localvar.sh"; echo "c27: [$in_sourced]"; }
lf; echo "c28: [${in_sourced-unset}]"
cat > "$tmp/status.sh" <<'EOF'
false
EOF
source "$tmp/status.sh"; echo "c29: last status of sourced file $?"
rm -rf "$tmp"
eval 'z=1; z=$((z+1))'; echo "c30: $z"
s='echo "c31: multi"; echo "c31: commands"'; eval "$s"
eval "echo c32: \$(echo nested)"
v="a b"; eval "w=\$v"; echo "c33: [$w]"
eval "arr2=(1 2 3)"; echo "c34: ${#arr2[@]}"
var_name="the_var"; printf -v "$var_name" '%s' "via printf"; echo "c35: $the_var"
eval 'return_test() { return 3; }'; return_test; echo "c36: $?"
eval "eval 'echo c37: double eval'"
