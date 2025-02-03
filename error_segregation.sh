#!/bin/bash

directory=$1

if [ -z "$directory" ]; then
  echo "Usage: $0 <path_to_directory>"
  exit 1
fi

mkdir -p "$directory/nbits"
mkdir -p "$directory/asan-deadlysignal"
mkdir -p "$directory/asan-heap-use"
mkdir -p "$directory/operand"
mkdir -p "$directory/compilation_error"
mkdir -p "$directory/other"

for file in "$directory"/*.stderr; do
  test_prefix=$(basename "$file" .stderr)

  if [ ! -s "$file" ]; then
    mv "$directory/$test_prefix."* "$directory/compilation_error/"
  else
    if grep -q "Unexpected nbits value @" "$file"; then
      mv "$directory/$test_prefix."* "$directory/nbits/"
    elif grep -q "AddressSanitizer: heap-use-after-free" "$file"; then
    mv "$directory/$test_prefix."* "$directory/asan-heap-use"
    elif grep -q "AddressSanitizer:DEADLYSIGNAL" "$file"; then
      mv "$directory/$test_prefix."* "$directory/asan-deadlysignal/"
    elif grep -q "Unexpected operand" "$file"; then
      mv "$directory/$test_prefix."* "$directory/operand/"
    else
      mv "$directory/$test_prefix."* "$directory/other/"
    fi
  fi
done

echo "Analysis complete."
