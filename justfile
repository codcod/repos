# https://just.systems

@_:
   just --list

# Build the main binary
[group('lifecycle')]
build:
    cargo build --release

# Build the plugins
[group('lifecycle')]
build-plugins:
    cargo build --release -p repos-health

# Run tests
[group('qa')]
test:
    cargo test

# Run coverage
[group('qa')]
coverage:
    cargo tarpaulin --skip-clean

# Registered plugins are binaries named `repos-*` in /usr/local/bin
# sudo ln -sf $(pwd)/target/release/repos-health /usr/local/bin/repos-health
#
# List available registered plugins
[group('run')]
list-plugins:
    ls -al /usr/local/bin/repos-* || echo "No plugins installed"

# vim: set filetype=Makefile ts=4 sw=4 et:
