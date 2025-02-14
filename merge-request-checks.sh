#!/bin/bash

# Stop the script if any command exits with a non-zero return code
set -e

# Run static code checks
cargo fmt --check
cargo clippy -- --deny warnings

# Execute tests and calculate the code coverage both as lcov and HTML report
rm -f target/llvm-cov/tests.lcov
mkdir -p target/llvm-cov/
cargo llvm-cov --no-cfg-coverage --all-features --ignore-filename-regex 'tests?\.rs' --lcov --output-path target/llvm-cov/tests.lcov

# Use diff-cover (https://github.com/Bachmann1234/diff_cover) and output code coverage compared to main branch
mkdir -p target/llvm-cov/html/
diff-cover target/llvm-cov/tests.lcov --html-report target/llvm-cov/html/patch.html
echo "HTML report available at $PWD/target/llvm-cov/html/patch.html"