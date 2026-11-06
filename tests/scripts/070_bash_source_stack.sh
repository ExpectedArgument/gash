# FUNCNAME, BASH_SOURCE, BASH_LINENO and caller across functions and sourced files, as in bash:
# a function's BASH_SOURCE[0] is the file it was defined in, and a sourced file gets a frame.
t=$(mktemp -d)
cd "$t" || exit 1
cat > lib.sh <<'EOF'
echo "lib top: FUNCNAME=(${FUNCNAME[*]}) SOURCE=(${BASH_SOURCE[*]}) LINENO=(${BASH_LINENO[*]})"
libf() {
    echo "libf: FUNCNAME=(${FUNCNAME[*]}) SOURCE=(${BASH_SOURCE[*]}) LINENO=(${BASH_LINENO[*]})"
    echo "libf caller 0: $(caller 0)"
    echo "libf caller 1: $(caller 1)"
}
inner_source() { . ./lib2.sh; }
EOF
cat > lib2.sh <<'EOF'
echo "lib2 top: FUNCNAME=(${FUNCNAME[*]}) SOURCE=(${BASH_SOURCE[*]}) LINENO=(${BASH_LINENO[*]})"
EOF
cat > main.sh <<'EOF'
#!/bin/bash
echo "main top: FUNCNAME=(${FUNCNAME[*]}) SOURCE=(${BASH_SOURCE[*]}) LINENO=(${BASH_LINENO[*]})"
. ./lib.sh
mainf() { libf; }
mainf
inner_source
echo "after: FUNCNAME=(${FUNCNAME[*]}) SOURCE=(${BASH_SOURCE[*]})"
[[ "${BASH_SOURCE[0]}" == "$0" ]] && echo "run, not sourced"
EOF
chmod +x main.sh
./main.sh
cd / && rm -rf "$t"
