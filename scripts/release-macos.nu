#!/usr/bin/env nu

let binary = 'rabbitmqadmin'
let src = $env.SRC | path expand
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

print $'Building on macOS in ($src)...'
build-with-cargo


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
cp -v $binary $'($release_dir)/($artifact_filename)'

print $'Release artifact at ($artifact_filename) is ready'
print $"Release directory: ($release_dir)"
ls $release_dir | print

echo $'artifact=($artifact_filename)' | save --append $env.GITHUB_OUTPUT

def 'build-with-cargo' [] {
  cargo rustc -q --bin $binary --target $target --release
}