# shift, set --, positional parameters, set -o / shopt queries
set -- a b c d e
echo "c1: $# $1"
shift
echo "c2: $# $1"
shift 2
echo "c3: $# $1 $*"
shift 5; echo "c4: status $? $#"
shift 0; echo "c5: $# $1"
shift 2; echo "c6: $# [$*]"
shift; echo "c7: status $? $#"
set -- "x y" z
echo "c8: $# [$1]"
set --
echo "c9: $#"
set -- a b c d e f g h i j k
echo "c10: $10 ${10} ${11}"
echo "c11: ${!#} ${@: -1}"
args=("$@")
echo "c12: ${#args[@]}"
set -- "${args[@]:3:2}"
echo "c13: $*"
while [ $# -gt 0 ]; do echo "c14: $1"; shift; done
f() { shift; echo "c15: in func $# $*"; }
set -- out1 out2
f a b c
echo "c16: outer still $# $*"
set -- -x -- -y
echo "c17: $# $1"
set -- -- a
echo "c18: $# $1"
set a b c
echo "c19: $# $2"
set -- "$@" d
echo "c20: $*"
set -- "${@:1:1}" "${@:3}"
echo "c21: $*"
IFS=: ; set -- $(echo "p:q:r"); unset IFS
echo "c22: $# $2"
set -f; set -- *; set +f
echo "c23: $1"
set -o | grep -E '^(errexit|nounset|pipefail|noglob)[[:space:]]' | tr -s ' \t' ' ' | sed 's/^/c24: /'
set -o errexit; [[ $- == *e* ]] && echo "c25: e in \$-"; set +o errexit
[[ $- == *e* ]] || echo "c26: e cleared"
set -o noglob; echo "c27:" *; set +o noglob
shopt -s nullglob; shopt nullglob | tr -s ' \t' ' ' | sed 's/^/c28: /'; shopt -u nullglob
shopt -q nullglob; echo "c29: $?"
shopt -s extglob nocasematch; shopt -q extglob && shopt -q nocasematch && echo "c30: multiple shopt"
shopt -u extglob nocasematch
set -u; [[ $- == *u* ]] && echo "c31: u in \$-"; set +u
set -- 1 2 3
shift $(( $# - 1 )); echo "c32: $1"
set -- "a b" "c d"
for p in "$@"; do echo "c33: [$p]"; done
count_args() { echo $#; }
echo "c34: $(count_args "$@") $(count_args $@) $(count_args "$*")"
set -- one two three
echo "c35: ${@:2:1} ${*:1:2}"
set -o pipefail; set -o | grep -E '^pipefail[[:space:]]' | tr -s ' \t' ' ' | sed 's/^/c36: /'; set +o pipefail
shopt -p nullglob | sed 's/^/c37: /'
set +o | grep -E 'noclobber$' | sed 's/^/c38: /'
