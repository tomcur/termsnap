#!/usr/bin/env bash

set -e

# This script expects to be run from the repository's root directory.

cargo build --release

export PATH="$PWD/target/release":$PATH

termsnap -o ./media/cow.svg -l 9 -c 28 -- cowsay Hello, world


(
    sleep 0.05
    echo -ne "for x in {16..231}; do printf \"\\\e[48;5;\${x}m%03d\\\e[0m \" \$x; done\r"
    sleep 0.05
) | termsnap -o ./media/colors.svg -l 16 -c 72 -- bash --noprofile --rcfile "$PWD/scripts/inputrc"

(
    sleep 0.05
    printf "echo \$-\r"
    sleep 0.05
    printf "tty\r"
    sleep 0.05
) | termsnap -o ./media/tty.svg -l 12 -c 60 -- bash --noprofile --rcfile "$PWD/scripts/inputrc"

termsnap -o ./media/nvim.svg -l 12 -c 60 -- nvim --clean ./scripts/example.py <<EOF
:set number
:syntax enable
:q
EOF
