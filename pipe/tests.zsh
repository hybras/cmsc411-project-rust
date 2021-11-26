tests="../tests"

for test in "$tests"/public*.mips; do
    name=$(basename -s .mips $test)
    # run my assembler
    cargo run --quiet --package assembler --bin assembler -- -i $test -o "$rs_tests/$name.asm"
    # run rust pipelined simulator
    cargo run --quiet --package pipe --bin pipe -- "$rs_tests/$name.asm" > "$rs_tests/$name.out"
    expected="$tests/$name.output"
    if diff -q $expected "$rs_tests/$name.out"
        rm "$rs_tests/$name.asm" $expected "$rs_tests/$name.out"
    fi
done