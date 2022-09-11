RUSTFLAGS="-C instrument-coverage" \
    LLVM_PROFILE_FILE=".tmp/json5format-%m.profraw" \
    cargo test --tests

llvm-profdata merge -sparse .tmp/json5format-*.profraw -o .tmp/json5format.profdata

llvm-cov report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-C instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --instr-profile=.tmp/json5format.profdata --ignore-filename-regex=/.cargo/registry --ignore-filename-regex=/rustc -show-region-summary


rm -r .tmp