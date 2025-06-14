find src -type f -name '*.rs' | while read -r file; do
  echo "$file"
  echo '```rust'
  cat "$file"
  echo '```'
done
