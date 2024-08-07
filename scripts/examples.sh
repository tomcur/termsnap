#!/usr/bin/env bash

set -e

# This script expects to be run from the repository's root directory.

cargo build --release

export PATH="$PWD/target/release":$PATH

termsnap -o ./media/cow.svg --lines 9 --columns 28 -- cowsay Hello, world

termsnap -o ./media/tokei.svg --lines 22 --columns 80 -- tokei

(
    # `sleep` gives bash time to be ready for the command, if this is omitted
    # the appearance of prompts can get messed up.
    sleep 0.05
    echo -ne "for x in {16..231}; do printf \"\\\e[48;5;\${x}m%03d\\\e[0m \" \$x; done\r"
    sleep 0.05
) | termsnap -o ./media/colors.svg --lines 16 --columns 72 -- bash --noprofile --rcfile "$PWD/scripts/inputrc"

(
    sleep 0.05
    printf "echo \$-\r"
    sleep 0.05
    printf "tty\r"
    sleep 0.05
) | termsnap -o ./media/tty.svg --lines 12 --columns 60 -- bash --noprofile --rcfile "$PWD/scripts/inputrc"

# On exit, for some terminals, Neovim clears the terminal screen by swapping
# back to the main terminal screen buffer. The `--render-before-clear` argument
# renders the terminal screen as it was just prior to that swap occurring.
termsnap -o ./media/nvim.svg --lines 12 --columns 60 --term xterm-256color --render-before-clear -- nvim --clean ./scripts/example.py <<EOF
:set number
:syntax enable
:q
EOF
