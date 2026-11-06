# Options on a script's #! line apply to the script (#3), also when it runs in-process.
t=$(mktemp -d)
cd "$t" || exit 1

printf '#!/bin/bash -e\nfalse\necho CONTINUED\n' > s1
printf '#!/bin/bash -u\necho "[$nope]"\necho CONTINUED\n' > s2
printf '#!/bin/bash -eu\nfalse\necho CONTINUED\n' > s3
printf '#!/usr/bin/env -S bash -e\nfalse\necho CONTINUED\n' > s4
printf '#!/bin/sh -e\nfalse\necho CONTINUED\n' > s5
printf '#!/bin/bash +e\nfalse\necho errexit-off\n' > s6
printf '#!/bin/bash -e\nset +e\nfalse\necho turned-off\n' > s7
printf '#!/bin/bash -o pipefail\necho not-reached\n' > s8
printf '#!/bin/bash -u\necho "args: $1 $2"\necho "$3"\necho CONTINUED\n' > s9
printf '#!/bin/bash -x\necho traced\n' > sx
chmod +x s1 s2 s3 s4 s5 s6 s7 s8 s9 sx

for s in ./s1 ./s2 ./s3 ./s4 ./s5 ./s6 ./s7 ./s8; do
    "$s" 2>/dev/null
    echo "$s: rc=$?"
done
./s9 one two 2>/dev/null
echo "./s9: rc=$?"
./sx 2>&1
echo "./sx: rc=$?"

cd / && rm -rf "$t"
