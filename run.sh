#!/bin/bash
cargo build
for source in examples/cases/*; do
  name=$(basename "$source")
  stem=${name%.*}
  target/debug/crqa -f "$source" -o "reports/cases/$stem.md" || test $? -eq 1
done
target/debug/crqa -d examples/cases --yaml -o reports/cases/all_cases.yaml || test $? -eq 1