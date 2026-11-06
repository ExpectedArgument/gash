#!/usr/bin/env bash
# Realistic: tiny INI parser into an associative array
tmp=$(mktemp -d) || exit 1
cd "$tmp" || exit 1

cat > config.ini <<'EOF'
; top-level comment
name = demo app
version=2.1

[database]
host = localhost
port = 5432
  user =   admin
# hash comment
password = "p@ss w0rd"

[paths]
root = /srv/app
logs = ${root}/logs
empty =
this line is garbage

[database]
pool = 10
EOF
printf 'last = no trailing newline' >> config.ini

declare -A cfg
declare -a sections=()
section=global
lineno=0
errors=0
while IFS= read -r line || [[ -n $line ]]; do
  (( lineno++ ))
  # strip leading/trailing whitespace
  line="${line#"${line%%[![:space:]]*}"}"
  line="${line%"${line##*[![:space:]]}"}"
  [[ -z $line || $line == \;* || $line == \#* ]] && continue
  if [[ $line =~ ^\[([A-Za-z0-9_]+)\]$ ]]; then
    section=${BASH_REMATCH[1]}
    seen=0
    for s in "${sections[@]}"; do [[ $s == "$section" ]] && seen=1; done
    (( seen )) || sections+=("$section")
    continue
  fi
  if [[ $line == *=* ]]; then
    key=${line%%=*}; val=${line#*=}
    key="${key%"${key##*[![:space:]]}"}"
    val="${val#"${val%%[![:space:]]*}"}"
    if [[ $val == \"*\" ]]; then val=${val:1:${#val}-2}; fi
    cfg[$section.$key]=$val
  else
    echo "warning: line $lineno unparsable: '$line'"
    (( errors++ ))
  fi
done < config.ini

get() { local k=$1 d=${2-}; if [[ -v cfg[$k] ]]; then printf '%s\n' "${cfg[$k]}"; else printf '%s\n' "$d"; fi; }

echo "sections: ${sections[*]}"
echo "keys: ${#cfg[@]}"
for k in "${!cfg[@]}"; do printf '%-20s = [%s]\n' "$k" "${cfg[$k]}"; done | sort
echo "db host: $(get database.host)"
echo "db port+1: $(( $(get database.port) + 1 ))"
echo "missing: $(get database.timeout 30)"
echo "empty: [$(get paths.empty x)]"
root=$(get paths.root)
logs=$(get paths.logs); logs=${logs//'${root}'/$root}
echo "logs: $logs"
echo "errors: $errors lines: $lineno"
for sec in "${sections[@]}"; do
  n=0
  for k in "${!cfg[@]}"; do [[ $k == "$sec".* ]] && (( n++ )); done
  echo "section $sec has $n keys"
done
cd .. && rm -rf "$tmp"
