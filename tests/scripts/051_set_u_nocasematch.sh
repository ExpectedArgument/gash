# set -u (nounset) in subshells, nocasematch and nocaseglob
( set -u; echo "c1: $undefined_var"; echo "c1: not reached" ) 2>/dev/null; echo "c1: status $?"
( set -u; echo "c2: ${undefined_var:-default}" )
( set -u; echo "c3: ${undefined_var-dash}" )
( set -u; echo "c4: ${#undefined_var}" ) 2>/dev/null; echo "c4: status $?"
( set -u; arr=(); echo "c5: ${#arr[@]}" )
( set -u; arr=(); echo "c6: [${arr[@]}]" ) 2>/dev/null; echo "c6: status $?"
( set -u; set --; echo "c7: [$@] [$*] $#" ) 2>/dev/null; echo "c7: status $?"
( set -u; echo "c8: $1" ) 2>/dev/null; echo "c8: status $?"
( set -u; x=""; echo "c9: empty ok [$x]" )
( set -u; declare -A h=(); echo "c10: ${h[nokey]-none}" )
( set -u; declare -A h=(); echo "c11: ${h[nokey]}" ) 2>/dev/null; echo "c11: status $?"
( set -u; echo "c12: $(( undefined_arith + 1 ))" ) 2>/dev/null; echo "c12: status $?"
( set -u; [[ -v undefined_var ]] || echo "c13: -v safe" )
( set -u; local_f() { local v; echo "c14: ${v-unset-local}"; }; local_f )
( set -u; u=1; unset u; echo "c15: $u" ) 2>/dev/null; echo "c15: status $?"
( set -u; a=(1 2); echo "c16: ${a[5]-oob}" )
( set -u; a=(1 2); echo "c17: ${a[5]}" ) 2>/dev/null; echo "c17: status $?"
( set -u; f() { echo "c18: $#"; }; f )
( set -u; echo "c19: ${undefined_var:+alt}|" )
( set -u; : "${undefined_var:=assigned}"; echo "c20: $undefined_var" )
( set -u; echo "c21: before"; echo "$nope" ; echo "c21: after" ) 2>/dev/null; echo "c21: status $?"
( set -u; for i in 1; do echo "c22: $i ${BASH_REMATCH[*]-}"; done )
shopt -s nocasematch
[[ ABC == abc ]]; echo "c23: $?"
[[ ABC == a* ]]; echo "c24: $?"
case HELLO in hello) echo "c25: case insensitive";; esac
case Yes in [y]es) echo "c26: bracket insensitive";; esac
[[ "ABC" != abc ]]; echo "c27: $?"
[[ aBc =~ ^ABC$ ]]; echo "c28: regex $?"
shopt -u nocasematch
[[ ABC == abc ]]; echo "c29: $?"
case HELLO in hello) echo no;; *) echo "c30: sensitive again";; esac
tmp=$(mktemp -d) || exit 1
: > "$tmp/Readme.TXT"
shopt -s nocaseglob
( cd "$tmp" && echo "c31:" *.txt )
shopt -u nocaseglob
( cd "$tmp" && echo "c32:" *.txt )
rm -rf "$tmp"
x="Hello"
echo "c33: ${x/hello/bye}"
shopt -s nocasematch
echo "c34: ${x/hello/bye}"
shopt -u nocasematch
