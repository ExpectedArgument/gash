# MSYS programs writing to a file opened with >> (issue #1: their writes failed
# with "Bad file descriptor", or were lost silently, because the file handle
# was append-only)
d=$(mktemp -d)
cd "$d" || exit 1
echo a > in.txt
t() { rm -f f; eval "$1"; echo "rc=$? [$(tr '\n' '|' < f)] $1"; }
t '/usr/bin/echo hi >> f'
t 'echo hi | /usr/bin/sed s/h/H/ >> f'
t '/usr/bin/sed s/a/b/ <<< abc >> f'
t '/usr/bin/grep -c . <<< x >> f'
t "/usr/bin/awk 'BEGIN{print 1}' >> f"
t '/usr/bin/sort <<< z >> f'
t '/usr/bin/cat <<< z >> f'
t '{ /usr/bin/echo x; } >> f'
t '( /usr/bin/echo x ) >> f'
t '/usr/bin/ls /nonexistent 2>> f'
# stdout and stderr to the same file must not overwrite each other
t '/usr/bin/sh -c "echo O1; echo E1 >&2; echo O2" >> f 2>&1'
t '/usr/bin/sh -c "echo O1; echo E1 >&2; echo O2" &>> f'
# an MSYS program started by a native one (as in a git hook or alias)
t 'git -c alias.z="!sed s/a/b/ in.txt" z >> f'
# an MSYS interpreter from a #! line
printf '#!/usr/bin/awk -f\nBEGIN { print "awk-script" }\n' > s.awk
chmod +x s.awk
t './s.awk >> f'
# appending keeps what is there, and builtin and MSYS writes land in order
echo start > g
{ echo a; /usr/bin/echo b; echo c; /usr/bin/printf 'd\n'; } >> g
/usr/bin/echo e >> g; echo f >> g
/usr/bin/cat g
# several writers under one redirection: pipeline stages, and a background
# job that writes after the shell and a foreground program have
rm -f p
( exec >> p 2>&1
  { /usr/bin/echo pipe-1; /usr/bin/ls /nonexistent; /usr/bin/echo pipe-2; } | /usr/bin/cat
  /usr/bin/sh -c 'while [ ! -e go ]; do sleep 0.05; done; echo bg-late' &
  echo builtin-1; /usr/bin/echo msys-fg; : > go; wait; echo builtin-2 )
/usr/bin/cat p
cd / && rm -rf "$d"
