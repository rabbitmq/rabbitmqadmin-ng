#!/usr/bin/env nu

let binary = 'rabbitmqadmin'
let src = $env.SRC | path expand
let os = $env.OS
let target = $env.TARGET

let version = (open Cargo.toml | get package.version)
let release_dir = $'($env.SRC)/target/($target)/release' | path expand
let executable = $'($release_dir)/($binary)'

print $'Packaging ($binary) v($version) for ($target) in ($src)...'
print $'Executable path is ($executable)...'

if not ('Cargo.lock' | path exists) {
  cargo generate-lockfile
}

rm -rf $release_dir
mkdir $release_dir

print $'Building on Linux in ($src)...'

#
# Linux
#

if ($os | str starts-with 'ubuntu') {
  print $"Building on Ubuntu \(($os)\)..."
  if $target == 'x86_64-unknown-linux-gnu' {
    build-with-cargo
  } else if $target == 'aarch64-unknown-linux-gnu' {
    build-with-cargo
  } else if $target == 'x86_64-unknown-linux-musl' {
    sudo apt-get update
    sudo apt-get install -y musl-tools
    build-static-with-cargo
  } else if $target == 'aarch64-unknown-linux-musl' {
    sudo apt-get update
    sudo apt-get install -y musl-tools
    build-static-with-cargo
  } else if $target == 'armv7-unknown-linux-gnueabihf' {
    sudo apt-get install pkg-config gcc-arm-linux-gnueabihf -y
    build-with-cargo
  } else {
    build-with-cargo
  }
}

if ($os | str starts-with 'fedora') {
  print $"Building on Fedora \(($os)\)..."
  if $target == 'aarch64-unknown-linux-gnu' {
    sudo dnf install -y gcc-aarch64-linux-gnu
    build-with-cargo
  } else if $target == 'armv7-unknown-linux-gnueabihf' {
    sudo dnf install pkg-config gcc-arm-linux-gnueabihf -y
    build-with-cargo
  } else {
    build-with-cargo
  }
}


#
# Release packaging
#

cd $src

print $"Release directory: ($release_dir)"
ls $release_dir | print

cp -r LICENSE* $release_dir
cp -r README* $release_dir

cd $release_dir

let artifact_filename = $'($binary)-($version)-($target)'

print $"Renaming release artifact to ($artifact_filename)..."
chmod +x $binary
cp -v $binary $'($release_dir)/($artifact_filename)'

print $'Release artifact at ($artifact_filename) is ready'
print $"Release directory: ($release_dir)"
ls $release_dir | print

echo $'artifact=($artifact_filename)' | save --append $env.GITHUB_OUTPUT

def 'build-with-cargo' [] {
  cargo rustc --bin $binary --target $target --release
}

def 'build-static-with-cargo' [] {
  $env.RUSTFLAGS = '-C target-feature=+crt-static'
  # Disable native-tls feature for musl builds (no OpenSSL dependency)
  cargo rustc --bin $binary --target $target --release --no-default-features
}
