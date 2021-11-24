tests_dir="../project/public_tests"

for test in "$tests_dir"/public*.asm; do
    name=$(basename -s .asm $test)
    cargo run --quiet --package small --bin small -- $test > "$name".output
    ../project/mips-small $test > "$tests_dir"/../"$name".output
    echo "TEST: $name"
    if diff -q "$tests_dir"/../"$name".output "$name".output; then
        rm "$tests_dir"/../"$name".output "$name".output
    fi
done