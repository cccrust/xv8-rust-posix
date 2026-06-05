#!/bin/bash
set -euo pipefail

rustc=$1
shift

source_file=""
target_triple=""
next_is_target=0

for arg in "$@"; do
    if [[ $next_is_target -eq 1 ]]; then
        target_triple=$arg
        next_is_target=0
        continue
    fi

    case "$arg" in
        --target)
            next_is_target=1
            ;;
        --target=*)
            target_triple="${arg#--target=}"
            ;;
        *.rs)
            source_file=$arg
            ;;
    esac
done

# Only modify source files for riscv64 cross-compilation.
# Host builds pass through unchanged.
if [[ "$target_triple" == "riscv64gc-unknown-none-elf" ]] && [[ "$source_file" == *"/src/bin/"*.rs ]]; then
    base_src=$(basename "$source_file")
    temp_dir="/tmp/rustc-wrapper-$$"
    mkdir -p "$temp_dir"
    temp_source="$temp_dir/$base_src"

    awk '
    NR == 1 {
        print "#![cfg_attr(target_arch = \"riscv64\", no_main)]"
        print "#![allow(dead_code, unused)]"
    }
    !main_injected && /^[[:space:]]*fn main[[:space:]]*\(/ {
        print "#[cfg_attr(target_arch = \"riscv64\", unsafe(no_mangle))]"
        main_injected = 1
    }
    { print }
    ' "$source_file" > "$temp_source"

    trap 'rm -rf "$temp_dir"' EXIT

    rewritten_args=()
    for arg in "$@"; do
        if [[ "$arg" == "$source_file" ]]; then
            rewritten_args+=("$temp_source")
        else
            rewritten_args+=("$arg")
        fi
    done

    exec "$rustc" "${rewritten_args[@]}"
fi

exec "$rustc" "$@"