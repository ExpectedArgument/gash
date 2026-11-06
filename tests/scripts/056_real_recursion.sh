#!/usr/bin/env bash
# Realistic: recursive algorithms (fibonacci w/ memo, hanoi, permutations, ackermann, gcd)
declare -A memo
fib() {
  local n=$1
  if (( n < 2 )); then REPLY=$n; return; fi
  if [[ -v memo[$n] ]]; then REPLY=${memo[$n]}; return; fi
  fib $((n - 1)); local a=$REPLY
  fib $((n - 2)); local b=$REPLY
  memo[$n]=$((a + b))
  REPLY=${memo[$n]}
}
for n in 0 1 2 10 20 50 90; do fib $n; echo "fib($n) = $REPLY"; done
echo "memo entries: ${#memo[@]}"

moves=0
hanoi() {
  local n=$1 from=$2 to=$3 via=$4
  (( n == 0 )) && return
  hanoi $((n - 1)) "$from" "$via" "$to"
  (( moves++ ))
  printf 'move %2d: disk %d %s -> %s\n' "$moves" "$n" "$from" "$to"
  hanoi $((n - 1)) "$via" "$to" "$from"
}
hanoi 4 A C B
echo "total moves: $moves (expected $(( (1 << 4) - 1 )))"

permute() {
  local prefix=$1 rest=$2
  if [[ -z $rest ]]; then printf '%s ' "$prefix"; return; fi
  local i
  for (( i = 0; i < ${#rest}; i++ )); do
    permute "$prefix${rest:i:1}" "${rest:0:i}${rest:i+1}"
  done
}
echo "perms: $(permute "" abcd)"

ack() {
  local m=$1 n=$2
  if (( m == 0 )); then REPLY=$((n + 1))
  elif (( n == 0 )); then ack $((m - 1)) 1
  else ack "$m" $((n - 1)); ack $((m - 1)) "$REPLY"
  fi
}
ack 2 3; echo "ackermann(2,3) = $REPLY"

gcd() { if (( $2 == 0 )); then echo "$1"; else gcd "$2" $(( $1 % $2 )); fi; }
echo "gcd(1071,462) = $(gcd 1071 462)"

fact() { if (( $1 <= 1 )); then REPLY=1; else fact $(( $1 - 1 )); REPLY=$(( $1 * REPLY )); fi; }
fact 20; echo "20! = $REPLY"

sum_digits() { local n=$1; if (( n < 10 )); then echo "$n"; else echo $(( n % 10 + $(sum_digits $(( n / 10 ))) )); fi; }
echo "digit sum of 987654321 = $(sum_digits 987654321)"

flatten() {
  local item
  for item in "$@"; do
    if [[ $item == *,* ]]; then
      local IFS=,
      flatten $item
    else
      printf '[%s]' "$item"
    fi
  done
}
echo "flatten: $(flatten a b,c d,e,f g)"

depth() { local d=$1; (( d >= 100 )) && { echo "reached depth $d"; return; }; depth $(( d + 1 )); }
depth 0

binsearch() {
  local -n arr=$1; local target=$2 lo=${3:-0} hi=${4:-$(( ${#arr[@]} - 1 ))}
  (( lo > hi )) && { echo -1; return; }
  local mid=$(( (lo + hi) / 2 ))
  if (( arr[mid] == target )); then echo "$mid"
  elif (( arr[mid] < target )); then binsearch "$1" "$target" $((mid + 1)) "$hi"
  else binsearch "$1" "$target" "$lo" $((mid - 1))
  fi
}
sorted=(2 3 5 7 11 13 17 19 23 29)
for t in 2 13 29 4; do echo "binsearch $t -> $(binsearch sorted $t)"; done
