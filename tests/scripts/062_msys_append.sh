# MSYS programs writing to a file opened with >> (issue #1: their writes failed
# with "Bad file descriptor", or were lost silently, because the file handle
# was append-only)
d=$(mktemp -d)
t() { rm -f "$d/f"; eval "$1"; echo "rc=$? size=$(wc -c < "$d/f") | $1"; }
t '/usr/bin/echo hi >> "$d/f"'
t 'echo hi | /usr/bin/sed s/h/H/ >> "$d/f"'
t '/usr/bin/sed s/a/b/ <<< abc >> "$d/f"'
t '/usr/bin/grep -c . <<< x >> "$d/f"'
t "/usr/bin/awk 'BEGIN{print 1}' >> \"\$d/f\""
t '/usr/bin/sort <<< z >> "$d/f"'
t '/usr/bin/cat <<< z >> "$d/f"'
t '{ /usr/bin/echo x; } >> "$d/f"'
t '( /usr/bin/echo x ) >> "$d/f"'
t '/usr/bin/ls /nonexistent 2>> "$d/f"'
# appending keeps what is there, and builtin and MSYS writes land in order
echo start > "$d/g"
{ echo a; /usr/bin/echo b; echo c; /usr/bin/printf 'd\n'; } >> "$d/g"
/usr/bin/echo e >> "$d/g"; echo f >> "$d/g"
/usr/bin/cat "$d/g"
rm -rf "$d"
