<div align="center">

# Termsnap 📸

**Create SVGs from terminal output**

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/termsnap.svg)](https://crates.io/crates/termsnap)
[![Build status](https://github.com/tomcur/termsnap/workflows/CI/badge.svg)](https://github.com/tomcur/termsnap/actions)

</div>

Termsnap turns terminal output into vector graphics. It uses an in-memory
instance of [Alacritty](https://github.com/alacritty/alacritty) under the hood
to be compatible with many terminal control sequences.

## Usage

See `$ termsnap --help` for CLI usage documentation. For example, to run the
`ls` command and store the output in an SVG file, run:

```bash
$ termsnap -- ls -l > ./out.svg
```

## Examples

These examples are generated by [./scripts/examples.sh](scripts/examples.sh).
Note many of these examples send automated input to an interactive bash
session.

![Termsnap output of the cowsay command saying "hello world"](./media/cow.svg)

![Termsnap output of a dump of indexed terminal colors](./media/colors.svg)

![Termsnap output of example Python code viewed in Neovim](./media/nvim.svg)

![Termsnap output of some tty commands](./media/tty.svg)

## Installation

Install using Cargo:

```bash
$ cargo install termsnap

# Run ls
$ termsnap --columns 80 --lines 36 -- ls --color=always -l

# Run an interactive bash session
$ termsnap --interactive --out ./interactive-bash.svg -- bash
```

Run using Nix flakes:

```bash
# Run ls
$ nix run github:tomcur/termsnap -- --columns 80 --lines 36 -- ls --color=always -l

# Run an interactive bash session
$ nix run github:tomcur/termsnap -- --interactive --out ./interactive-bash.svg -- bash
```

## A note on fonts

The SVG generated by Termsnap assumes the font used is monospace with a glyph
width/height ratio of 0.60 and a font ascent of 0.75. The font is not
embedded and the text not converted to paths. If the client rendering the SVG
can't find the specified font, the SVG may render incorrectly, especially if
the used font's dimensions do not match Termsnap's assumptions. You can use,
e.g., Inkscape to convert the text to paths---the downside is the text may lose
crispness when rendering at low resolutions. You can also convert the SVG to a
raster image.

```bash
# Text to path
$ inkscape --export-text-to-path --export-plain-svg --export-filename=./out.svg ./in.svg

# Render to raster image
$ inkscape --export-width=800 --export-filename=./out.png ./in.svg
```
