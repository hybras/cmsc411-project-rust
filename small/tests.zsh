c_proj="../../c"
c_tests="$c_proj/public_tests"
rs_tests="../tests"

for test in "$c_tests"/public*.mips; do
    name=$(basename -s .mips $test)
    # run my assembler
    cargo run --quiet --package assembler --bin assembler -- -i $test -o "$rs_tests/$name.asm"
    # run rust unpipelined simulator
    cargo run --quiet --package small --bin small -- "$rs_tests/$name.asm" > "$rs_tests/small-$name.out"
    # run unpipelined c simulator
    "$c_proj"/mips-small "$rs_tests/$name.asm" > "$c_tests/small-$name.out"
    if diff -q "$rs_tests/small-$name.out" "$c_tests/small-$name.out"; then
        rm "$rs_tests/$name.asm" "$rs_tests/small-$name.out" "$c_tests/small-$name.out"
    fi
done