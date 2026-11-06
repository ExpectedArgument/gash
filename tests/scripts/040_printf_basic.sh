# printf: conversions, widths, precision, format reuse
printf 'c1: %s\n' hello
printf 'c2: %s %s\n' a b c d
printf 'c3: [%s]\n' a b c
printf 'c4: %d %i\n' 42 -17
printf 'c5: [%5d] [%-5d] [%05d] [%+d] [% d]\n' 42 42 42 42 42
printf 'c6: %5.2f|%.3f|%f|%.1f\n' 3.14159 2.5 1 0.25
printf 'c7: [%-10s|] [%10s]\n' left right
printf 'c8: %x %X %o %#x %#o\n' 255 255 8 255 8
printf 'c9: %e %E %.2e\n' 12345.678 0.00012 1e10
printf 'c10: %g %g %g %g\n' 100000 1000000 0.0001 0.00001
printf 'c11: %c%c%c\n' abc def g
printf 'c12: [%*d] [%-*d]\n' 6 42 6 42
printf 'c13: %%literal%%\n'
printf 'c14: [%.3s] [%10.3s]\n' abcdef abcdef
printf 'c15: %s|\n'
printf 'c16: %d %d\n' 1 2 3
printf 'c17: %s=%d\n' a 1 b 2 c
printf 'c18: %d\n' "'A" '"B'
printf 'c19: %u\n' -1
printf 'c20: %d\n' 0x1F 010 -0
printf 'c21: tab\there\\back\n'
printf 'c22: \x41\101\n'
printf -- 'c23: -dash\n'
printf '%s\n' "c24: arg with %d not format"
printf 'c25: no newline'; echo
printf 'c26: %5s|%-5s|\n' toolong toolong
printf 'c27: %.2f %.2f\n' 1.005 2.675
printf 'c28: %d\n' 9223372036854775807
( printf 'c29: %d\n' abc ) 2>/dev/null; echo "c29: status $?"
( printf 'c30: %d\n' 12abc ) 2>/dev/null; echo "c30: status $?"
printf 'c31: [%s]\n' ''
printf 'c32: %s %s %s\n' 1 2
printf 'c33: %04.1f\n' 3.14159
printf 'c34: %+.2e\n' -1234.5
printf 'c35: [%8.3f]\n' -3.14159
printf 'c36: %x\n' -1
printf 'c37: %o\n' 0
printf 'c38: %s\n' 3.0
printf 'c39: [%-6.2f]\n' 1.5
printf 'c40: %.0f %.0f %.0f %.0f\n' 0.5 1.5 2.5 3.5
printf 'c41: %10.4f|\n' 2.71828
printf 'c42: %d%%\n' 75
printf 'c43: %s\n' "multiple words in one arg"
printf 'c44: %5s%5s\n' a b c
printf 'c45: %-3s|' x y z; echo
printf 'c46: %i\n' -0x10
printf 'c47: %.3e\n' 0
printf 'c48: %G\n' 0.000001234
printf 'c49: %f\n' -0.0
printf 'c50: %s %s\n' "$(printf '%03d' 5)" "$(printf '%x' 4095)"
