# Changelog

## [0.1.2] - 2024-06-28

### Features

- Print new lines after <rect> and <text> in generated SVG
- Only use non-breaking space if prev char was a regular space
- Reduce number of <text> elements in generated SVG
- Reduce number of background color rectangles in generated svg

### Documentation

- Remove unused example

### Refactor

- Explicitly forbid unsafe code

### Other

- Generate examples

## [0.1.1] - 2024-06-27

### Bug Fixes

- Correctly handle terminal resizing

### Documentation

- *(CHANGELOG)* Track all packages in a single changelog
- *(README)* Remove unnecessary badge

### Build System and CI

- Add release-plz
- Add rust build

