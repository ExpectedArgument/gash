# An argument starting with // is an escaped Windows switch: a native program gets it with one
# slash fewer, as with MSYS (`cmd //c`, `taskkill //F`), unless it looks like `//server/share`.
# Start from no conversion settings, whatever the environment exports (Claude Code sets one).
unset MSYS_NO_PATHCONV MSYS2_ARG_CONV_EXCL
git rev-parse --sq-quote //c //F ///x //server/share/x //c/x //
cmd //c echo switch passed
export MSYS_NO_PATHCONV=1
git rev-parse --sq-quote //c
unset MSYS_NO_PATHCONV
