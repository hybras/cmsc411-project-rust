c_proj="../../c"
c_tests="$c_proj/public_tests"rs_tests="../tests"

for test in "$c_tests"/public*.mips; do
    name=$(basename -s .mips $test)
    cargo run --quiet --package assembler --bin assembler -- -i $test -o "$rs_tests/$name.asm"
    "$c_proj/"asm $test "$c_tests/$name.asm"
    if diff -q "$rs_tests/$name.asm" "$c_tests/$name.asm"; then
        rm "$rs_tests/$name.asm" "$c_tests/$name.asm"
    fi
done