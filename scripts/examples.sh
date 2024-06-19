#!/usr/bin/env bash

# This script expects to be run from the repository's root directory.

cargo run -- -o ./media/ls.svg -l 12 -c 60 -- ls -l --color=always

cargo run -- -o ./media/nvim.svg -l 12 -c 60 -- nvim --clean \
    -c "set number" \
    -c "syntax enable" \
    -c "lua vim.defer_fn(function() vim.cmd('q') end, 0)" \
    ./scripts/example.py
