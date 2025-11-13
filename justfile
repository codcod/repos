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
    cargo build --release -p repos-validate

# Run tests
[group('qa')]
test:
    cargo test --quiet

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

[group('devex')]
link-plugins:
    sudo ln -sf $(pwd)/target/release/repos-health /usr/local/bin/repos-health
    sudo ln -sf $(pwd)/target/release/repos-health /usr/local/bin/repos-health

[group('devex')]
unlink-plugins:
    sudo rm -f /usr/local/bin/repos-health
    sudo rm -f /usr/local/bin/repos-validate

# vim: set filetype=Makefile ts=4 sw=4 et:
