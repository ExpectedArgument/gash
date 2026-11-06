#!/usr/bin/env bash
# Realistic: text report generator with printf tables, totals and bar charts
data=$(cat <<'EOF'
region|product|units|price_cents
north|widget|120|250
south|widget|80|250
north|gadget|15|1999
east|widget|200|240
south|gizmo|7|10050
east|gadget|30|1899
west|gizmo|0|9900
north|gizmo|3|10050
EOF
)

fmt_money() {
  local c=$1 neg=""
  (( c < 0 )) && { neg="-"; c=$(( -c )); }
  local d=$(( c / 100 )) r=$(( c % 100 )) s out=""
  s=$d
  while (( ${#s} > 3 )); do out=",${s: -3}$out"; s=${s:0:${#s}-3}; done
  printf -v REPLY '%s$%s%s.%02d' "$neg" "$s" "$out" "$r"
}

declare -A region_total product_units
total=0
rows=0
hr=$(printf '%.0s-' {1..52})
printf '%-8s %-8s %6s %12s %14s\n' REGION PRODUCT UNITS PRICE REVENUE
echo "$hr"
while IFS='|' read -r region product units price; do
  [[ $region == region ]] && continue
  rev=$(( units * price ))
  (( total += rev, rows++ ))
  (( region_total[$region] += rev ))
  (( product_units[$product] += units ))
  fmt_money "$price"; p=$REPLY
  fmt_money "$rev"; r=$REPLY
  printf '%-8s %-8s %6d %12s %14s\n' "$region" "$product" "$units" "$p" "$r"
done <<< "$data"
echo "$hr"
fmt_money "$total"
printf '%-8s %-8s %6s %12s %14s\n' TOTAL "($rows)" "" "" "$REPLY"
echo

echo "Revenue by region:"
max=0
for r in "${!region_total[@]}"; do (( region_total[$r] > max )) && max=${region_total[$r]}; done
while read -r r; do
  v=${region_total[$r]}
  pct10=$(( v * 1000 / total ))
  bar_len=$(( v * 30 / max ))
  bar=$(printf '%*s' "$bar_len" '')
  fmt_money "$v"
  printf '  %-6s %12s %3d.%d%% |%s\n' "$r" "$REPLY" $(( pct10 / 10 )) $(( pct10 % 10 )) "${bar// /#}"
done < <(printf '%s\n' "${!region_total[@]}" | sort)
echo

echo "Units by product (sorted by units desc):"
for p in "${!product_units[@]}"; do printf '%d %s\n' "${product_units[$p]}" "$p"; done | sort -rn | while read -r u p; do
  printf '  %-8s %5d\n' "$p" "$u"
done
echo

zero=$(while IFS='|' read -r region product units price; do [[ $units == 0 ]] && echo "$region/$product"; done <<< "$data")
echo "Zero-unit lines: ${zero:-none}"
avg=$(( total / rows ))
fmt_money "$avg"; echo "Average revenue per line: $REPLY"
fmt_money -12345; echo "Negative format check: $REPLY"
