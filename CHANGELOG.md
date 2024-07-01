# Changelog

## [0.2.0] - 2024-07-01

### Breaking
- *(lib)* [**breaking**] rename cols -> columns
([c0fe244](https://github.com/tomcur/termsnap/commit/c0fe2446fac97b01f8f367f872e15282a2d399a6))


### Features
- *(lib)* add methods to access terminal grid cells to Screen
([a1883d7](https://github.com/tomcur/termsnap/commit/a1883d75067fcc5cc38ff84988af5e76108b7a0a))
- implement rendering from ANSI-escaped data on stdin
([e603f97](https://github.com/tomcur/termsnap/commit/e603f97ef80d2c1c06e32bb20a58045b5b24c2d1))


### Bug Fixes
- separate main logic and CLI arguments consistency checks
([501e629](https://github.com/tomcur/termsnap/commit/501e6293849df204d8427e8888876a6feec0238f))


### Refactor
- explicitly pass around I/O handles, fixes test
([0f35410](https://github.com/tomcur/termsnap/commit/0f3541084faaa07d08ed0a33298ede352c38e10a))
- remove unused code
([adfb990](https://github.com/tomcur/termsnap/commit/adfb99049215377ad136541ff49333dee341ed32))


### Testing
- add test hitting the main code path
([fc27cd6](https://github.com/tomcur/termsnap/commit/fc27cd6e5f069a21483b234afcbaabec196a1ac5))
([c7e4149](https://github.com/tomcur/termsnap/commit/c7e4149756fb1a1b40f4350ed4e1f929ade66120))
([ff0d0e5](https://github.com/tomcur/termsnap/commit/ff0d0e5d8c9b35c8930353e465246d56bf9a2f62))
([be39b89](https://github.com/tomcur/termsnap/commit/be39b89c04357363cbf67b39e8cf67a4400f975b))


## [0.1.3] - 2024-06-29

### Bug Fixes
- correctly calculate NoHash
([84d60bf](https://github.com/tomcur/termsnap/commit/84d60bfc5b2d0c0f6a4d91b484e18161e847d8c8))


### Documentation
- *(README)* add headings to examples
([739f43e](https://github.com/tomcur/termsnap/commit/739f43e3bc61fc81d6e8ab2e56e21f7a3d3cc23a))
- *(README)* remove -o switch
([f43df9d](https://github.com/tomcur/termsnap/commit/f43df9d339f6e368326b6271ab236fd2b1b18c1f))
- *(README)* add Tokei example
([1259a74](https://github.com/tomcur/termsnap/commit/1259a74e4636497a4af37885fd91c0dcd0292612))
- *(README)* improve wording
([4850b72](https://github.com/tomcur/termsnap/commit/4850b7211634e211f038831ae9e6991a1436bfc6))
- *(README)* add note about sleep
([57006ce](https://github.com/tomcur/termsnap/commit/57006ce09b73a2098a0b30a016a5104b8b3a7d64))
- *(README)* put the nvim example before the colors example
([a9b421b](https://github.com/tomcur/termsnap/commit/a9b421b5245df75f6fe8b26a503e64d3c0482ec0))
- *(README)* add command to examples
([12680d7](https://github.com/tomcur/termsnap/commit/12680d7a960021a7635ec1011cb9a9d7791fa593))
- *(README)* improve cowsay example margins
([a5121c4](https://github.com/tomcur/termsnap/commit/a5121c454e41c4ad2cbfed694c1c1e947d7ca225))
- *(README)* simplify cowsay example
([0488ffb](https://github.com/tomcur/termsnap/commit/0488ffbfcfd749d4d546a88454b69b96ff3f80af))
- *(README)* simplify nvim example
([1b7a664](https://github.com/tomcur/termsnap/commit/1b7a66489f858caa4e5adeb8fe07cf8778e2f90b))
- *(README)* fix link to examples.sh
([cd3ee63](https://github.com/tomcur/termsnap/commit/cd3ee635604c31a3d20909beed2ef7805895943e))
- *(CHANGELOG)* add links to generated changelog
([9c88dd2](https://github.com/tomcur/termsnap/commit/9c88dd2bec5a269682f97992df50043b95dbf305))


### Other
- generate examples
([8300931](https://github.com/tomcur/termsnap/commit/8300931f64068714a967d47cb8ffa3f4e1301692))


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

