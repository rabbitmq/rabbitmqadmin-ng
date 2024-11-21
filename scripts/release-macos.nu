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
ls $release_dir

print "Compiling a release archive..."

let archive_filename = $'($binary)-($version)-($target).tar.gz'
print $'Release archive name: ($archive_filename)'
tar --verbose -czf $'($release_dir)/($archive_filename)' $binary
print $'Release archive at ($archive_filename) is ready'
echo $'archive=($archive_filename)' | save --append $env.GITHUB_OUTPUT

def 'build-with-cargo' [] {
  cargo rustc --bin $binary --target $target --release
}