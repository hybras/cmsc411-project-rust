tests_dir="../project/public_tests"

for test in "$tests_dir"/public*.mips; do
    local name=$(basename -s .mips $test)
    # cargo run --quiet --package assembler --bin assembler -- -i $test -o "$name".asm
    ../project/asm $test "$tests_dir"/"$name".asm
    # diff "$tests_dir"/"$name".asm "$name".asm
done