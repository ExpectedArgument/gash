# Here-doc and here-string input that an external program leaves unread stays for the next
# reader (#2), as with bash's pipe-backed here-documents.
t=$(mktemp -d)
cd "$t" || exit 1

printf '#!/usr/bin/env bash\n/usr/bin/true\ncat\n' > repro
chmod +x repro
./repro <<< data
./repro <<'EOF'
here-doc 1
here-doc 2
EOF
f() { x=$(/usr/bin/true); cat; }
f <<< function
g() { /usr/bin/true; cat; }
g <<< "from g"
h() { x=$(echo a); cat; }
h <<< builtin-cmdsub
printf 'from a file\n' > input
g < input

# A launcher that resolves a path with a tool first, then execs the real reader.
printf '#!/bin/bash\nd=$(cygpath -w /c/Windows)\nexec cat\n' > launcher
chmod +x launcher
./launcher <<< launched

# Builtins and external programs taking turns at reading the same input.
{ read -r a; /usr/bin/true; read -r b; cat; } <<'EOF'
1
2
3
4
EOF
echo "a=$a b=$b"
{ /usr/bin/head -n1; read -r c; echo "c=[$c]"; } <<< $'x\ny'
{ head -n1; cat; } <<EOF
1
2
3
EOF

# Input bigger than a pipe buffer arrives whole.
big=$(printf '%0100000d' 0)
{ /usr/bin/true; wc -c; } <<< "$big"

cd / && rm -rf "$t"
