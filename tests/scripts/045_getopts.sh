# getopts loops, silent mode, missing arguments, OPTIND
parse() {
  local OPTIND opt
  while getopts "ab:c" opt; do
    case $opt in
      a) echo "  a";;
      b) echo "  b=$OPTARG";;
      c) echo "  c";;
      \?) echo "  invalid [${OPTARG-unset}]";;
    esac
  done
  shift $((OPTIND-1))
  echo "  rest: $# [$*] OPTIND=$OPTIND"
}
echo c1; parse -a -b val -c file1 file2
echo c2; parse -ac -bval
echo c3; parse -b "with space" -- -a
echo c4; parse file -a
echo c5; parse -x -a 2>/dev/null
echo c6; parse -b 2>/dev/null
echo c7; parse
echo c8; parse -abX
silent() {
  local OPTIND opt
  while getopts ":ab:" opt; do
    case $opt in
      a) echo "  a";;
      b) echo "  b=$OPTARG";;
      :) echo "  missing arg for -$OPTARG";;
      \?) echo "  unknown -$OPTARG";;
    esac
  done
  echo "  OPTIND=$OPTIND"
}
echo c9; silent -a -z -b
echo c10; silent -b -a
echo c11; silent -- -a
echo c12; silent -ba
set -- -a -b gv rest
OPTIND=1
while getopts ab: o; do echo "c13: $o ${OPTARG-}"; done
shift $((OPTIND-1)); echo "c14: $*"
OPTIND=1
getopts a o -a; echo "c15: $? $o"
getopts a o -a; echo "c16: $? [$o]"
OPTIND=1
getopts a o nonopt; echo "c17: $? [$o]"
OPTIND=1
getopts a o -; echo "c18: $? [$o]"
OPTIND=1
getopts "a" o -q 2>/dev/null; echo "c19: $? [$o] [${OPTARG-unset}]"
OPTIND=1
getopts ":a" o -q; echo "c20: $? [$o] [$OPTARG]"
numeric() {
  local OPTIND opt
  while getopts "n:v" opt; do
    case $opt in
      n) echo "  n=$OPTARG";;
      v) echo "  v";;
    esac
  done
  echo "  next=${!OPTIND-none}"
}
echo c21; numeric -n 5 -v -n -3 target
echo c22; numeric -vn10
echo c23; numeric -n "" -v
reuse() { local OPTIND=1 o; while getopts "x" o; do echo "  got $o"; done; }
echo c24; reuse -x; reuse -x -x
