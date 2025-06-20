## [3.1.1](https://github.com/codcod/repos/compare/v3.1.0...v3.1.1) (2025-06-19)


### Bug Fixes

* cyclomatic complexity for java ([a4df51c](https://github.com/codcod/repos/commit/a4df51c493e4419bd60ee5e8c970a2aacd19c57e))

# [3.1.0](https://github.com/codcod/repos/compare/v3.0.0...v3.1.0) (2025-06-18)


### Bug Fixes

* add readme analysis to registry ([d848e28](https://github.com/codcod/repos/commit/d848e28f166ab40f2f51603ea96e8f6741f5bf22))
* build issues ([6117426](https://github.com/codcod/repos/commit/61174260f02131df22571a5b8812ec4e6234b053))
* dry-run option ([6daf56e](https://github.com/codcod/repos/commit/6daf56e9247cd18213f4296ecf7b0e9283bd683b))
* remove basic configuration option for health command ([56c85a0](https://github.com/codcod/repos/commit/56c85a069237f10ef05affbdb269f1e93e2e2ea5))


### Features

* add gen-config option ([628d1ff](https://github.com/codcod/repos/commit/628d1ff2ad8a73d5e0cd11e97e8ff1a85c641e3d))
* add list-categories option ([fbb7be0](https://github.com/codcod/repos/commit/fbb7be079a44128a87538a5b75e98740e3fbf8e4))

# [3.0.0](https://github.com/codcod/repos/compare/v2.1.0...v3.0.0) (2025-06-18)


### Bug Fixes

* build issues ([94a9e47](https://github.com/codcod/repos/commit/94a9e47253a89dd71041d28529809427d11442df))
* build issues ([e755132](https://github.com/codcod/repos/commit/e755132d4f1fac4c3aebfafaf1d72455c43a28a7))


### Features

* remove pipeline feature ([e5e5db1](https://github.com/codcod/repos/commit/e5e5db11444becaa5768ad5fbfe7519ba7b9ad27))
* remove profile feature ([696253c](https://github.com/codcod/repos/commit/696253cab0f397e10b9c8b58f22ff6f37c7aa081))


### BREAKING CHANGES

* Profile feature has been removed
* Pipeline feature has been removed

# [2.1.0](https://github.com/codcod/repos/compare/v2.0.0...v2.1.0) (2025-06-18)


### Features

* more comprehensive report ([f368bdb](https://github.com/codcod/repos/commit/f368bdbd6e01a1dc17e1238e2bf17c50cd32cd82))
* more readable report ([d815a7f](https://github.com/codcod/repos/commit/d815a7f4b461dba5affba81f263fd7af4df2e5f7))
* more readable report ([da47e0b](https://github.com/codcod/repos/commit/da47e0b74bdfd262cab45b9fe54a947963017055))
* yaml configuration is optional now ([f51e298](https://github.com/codcod/repos/commit/f51e298b8849a2522811e829c9e9dee2a899537d))

# [2.0.0](https://github.com/codcod/repos/compare/v1.5.0...v2.0.0) (2025-06-17)


### Features

* remove health command ([febeb85](https://github.com/codcod/repos/commit/febeb85ebee81d748245aff5a64efbcacc7cb89e))
* replace orchestrate command with health command ([a081578](https://github.com/codcod/repos/commit/a08157816c0bc62c21573c7820ff1a86e3bb91d2))


### BREAKING CHANGES

* The orchestrate command has been replaced with health command.

# [1.5.0](https://github.com/codcod/repos/compare/v1.4.0...v1.5.0) (2025-06-17)


### Features

* clean up migration files ([494a629](https://github.com/codcod/repos/commit/494a6293e3b42230d86859102b1b80403c94044a))
* remove feature flags entirely ([52e3202](https://github.com/codcod/repos/commit/52e32026f5abb413b4347dcc8b6bd8b57ec29efd))

# [1.4.0](https://github.com/codcod/repos/compare/v1.3.0...v1.4.0) (2025-06-16)


### Features

* more acurate cyclomatic complexity calculation ([4fe34d0](https://github.com/codcod/repos/commit/4fe34d078c2694cfac89ac801e190d94ba5417c6))

# [1.3.0](https://github.com/codcod/repos/compare/v1.2.2...v1.3.0) (2025-06-15)


### Bug Fixes

* resolve all linter issues in cyclomatic complexity checker ([65905bb](https://github.com/codcod/repos/commit/65905bbeb79964cb852062dcd59338924ffe1b91))


### Features

* add cyclomatic complexity checker ([da1e319](https://github.com/codcod/repos/commit/da1e31933917c067fde5cbcb6feec5aeee4c1bef))
* add repository health dashboard with multi-language support ([6826445](https://github.com/codcod/repos/commit/6826445cee93bd3341a720b9e7a74c6e5945af8a))

## [1.2.2](https://github.com/codcod/repos/compare/v1.2.1...v1.2.2) (2025-06-15)


### Bug Fixes

* ensure version information in github releases ([39d0cf2](https://github.com/codcod/repos/commit/39d0cf25dd7df359a8035d1be74156dd57dbc1ab))

## [1.2.1](https://github.com/codcod/repos/compare/v1.2.0...v1.2.1) (2025-06-15)


### Bug Fixes

* enable build-time version information with environment fallback ([c5e0bfc](https://github.com/codcod/repos/commit/c5e0bfc476b2eff1cf625bec971f0ad24dfe841f))

# [1.2.0](https://github.com/codcod/repos/compare/v1.1.0...v1.2.0) (2025-06-14)


### Bug Fixes

* add explicit yaml import aliases to resolve undefined yaml errors ([5bfcd54](https://github.com/codcod/repos/commit/5bfcd54b5bdc20607c01c642cacb584ffd74778f))
* add install-go-tools target and ensure goimports is available in ci ([26d461f](https://github.com/codcod/repos/commit/26d461ffc2a40b44049153adbf94735e4ed26847))
* replace broken gosec github action with direct tool installation ([7e5e7a0](https://github.com/codcod/repos/commit/7e5e7a0c7e1794d5627f22e7ada7bcd260d91400))
* resolve golangci-lint path issues in ci environment ([6113c1b](https://github.com/codcod/repos/commit/6113c1b10e4e000ee987824db4145f8367ee17df))
* update golangci-lint to latest version for go 1.24 compatibility ([4476dc0](https://github.com/codcod/repos/commit/4476dc04fa491f8d1c0f7cd6b76c1fd07dcc8a2f))


### Features

* a new release ([25776e5](https://github.com/codcod/repos/commit/25776e56b5e096c27d07afc66d70df9d0c23a45d))

# [1.1.0](https://github.com/codcod/repos/compare/v1.0.1...v1.1.0) (2025-06-14)


### Features

* remove max depth option ([5ba0c4a](https://github.com/codcod/repos/commit/5ba0c4a5624e49f316cc30bf5fad39d71fd55c42))

## [1.0.1](https://github.com/codcod/repos/compare/v1.0.0...v1.0.1) (2025-06-08)


### Bug Fixes

* **tests:** add tests ([c866b91](https://github.com/codcod/repos/commit/c866b91d9ee55130082da682a8b918c876f4b2f4))

# [1.0.0](https://github.com/codcod/repos/compare/v0.2.1...v1.0.0) (2025-06-08)


### Features

* migrate to semantic-release for automated releases ([5a4877f](https://github.com/codcod/repos/commit/5a4877f8521e1ff5b4ba50a8c874bd457598dedb))


### BREAKING CHANGES

* Version management is now handled by semantic-release.
Developers must use conventional commit messages for proper versioning.

# Changelog

All notable changes to this project will be documented in this file.

This file is automatically generated by [semantic-release](https://github.com/semantic-release/semantic-release).

## [0.2.1](https://github.com/codcod/repos/compare/...v0.2.1) (2024-12-19)

### Features

* Initial release with semantic-release integration
* CLI tool to manage multiple GitHub repositories
* Clone multiple repositories from config file
* Filter repositories by tag
* Run commands in all or filtered repositories
* Create pull requests for changes
* Parallel execution support
* Real-time, colorized logs
* Per-repo log files

### Bug Fixes

* Improved error handling and logging
* Better version management with semantic-release

### Documentation

* Added comprehensive README with usage examples
* Added installation instructions via Homebrew
