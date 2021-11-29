tests="../tests"

for test in "$tests"/public*.mips; do
    name=$(basename -s .mips $test)
    echo $name
    # run my assembler
    cargo run --quiet --package assembler --bin assembler -- -i $test -o "$tests/$name.asm"
    # run rust pipelined simulator
    cargo run --quiet --package pipe --bin pipe -- "$tests/$name.asm" > "$tests/$name.out"
    expected="$tests/$name.output"
    if diff -q $expected "$tests/$name.out"; then
        rm "$tests/$name.asm" "$tests/$name.out"
    fi
done