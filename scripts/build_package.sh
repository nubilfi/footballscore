#!/bin/bash

VERSION="$1"
RELEASE="$2"

# Ensure Python symlink exists
if [ ! -e /usr/bin/python ]; then
    ln -s /usr/bin/python3 /usr/bin/python
fi

# Assuming cargo is installed, set up the environment
. ~/.cargo/env

# Build the Rust program
cargo build --release

# Create the PKGBUILD file
echo "pkgname=footballscore" > PKGBUILD
echo "pkgver=${VERSION}" >> PKGBUILD
echo "pkgrel=${RELEASE}" >> PKGBUILD
echo "pkgdesc='CLI Football Score API'" >> PKGBUILD
echo "arch=('x86_64')" >> PKGBUILD
echo "license=('MIT')" >> PKGBUILD
echo "depends=('python' 'cargo')" >> PKGBUILD  # Add necessary dependencies

echo "package() {" >> PKGBUILD
echo "    install -Dm755 ../target/release/footballscore \$pkgdir/usr/bin/footballscore" >> PKGBUILD
echo "}" >> PKGBUILD

# Create the package using makepkg
makepkg -si

