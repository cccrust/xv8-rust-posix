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

if [[ "$source_file" == *"/src/bin/"*.rs ]]; then
    temp_source=$(mktemp /tmp/rustc-wrapper.XXXXXX)

    if [[ "$target_triple" == "riscv64gc-unknown-none-elf" ]]; then
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
    else
        awk '
        NR == 1 {
            print "#![allow(dead_code, unused)]"
        }
        { print }
        ' "$source_file" > "$temp_source"
    fi

    trap 'rm -f "$temp_source"' EXIT

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