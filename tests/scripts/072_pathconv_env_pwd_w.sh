# MSYS_NO_PATHCONV and MSYS2_ARG_CONV_EXCL count when exported or given as a command prefix,
# as in Git Bash. pwd -W prints the current directory in Windows form.
git rev-parse --sq-quote /c/x
MSYS_NO_PATHCONV=1 git rev-parse --sq-quote /c/x
MSYS_NO_PATHCONV=1
git rev-parse --sq-quote /c/x
export MSYS_NO_PATHCONV
git rev-parse --sq-quote /c/x
unset MSYS_NO_PATHCONV
MSYS2_ARG_CONV_EXCL=/c git rev-parse --sq-quote /c/x /d/y
export MSYS2_ARG_CONV_EXCL='/d;/e'
git rev-parse --sq-quote /c/x /d/y /e/z
unset MSYS2_ARG_CONV_EXCL

cd /c/Windows/System32 || exit 1
pwd -W
pwd -LW
pwd -WP
pwd -P
pwd -L
pwd -X 2>/dev/null
echo "pwd -X: rc=$?"
