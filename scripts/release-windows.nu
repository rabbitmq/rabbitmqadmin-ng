#!/usr/bin/env nu

const binary = 'rabbitmqadmin'
const binary_filename = $'($binary).exe'
let src = $env.SRC | path expand
let target = $env.TARGET | default $"x86_64-pc-windows-msvc"

let version = (open Cargo.toml | get package.version)
let release_dir = $'($env.SRC)/target/($target)/release' | path expand
let executable = $'($release_dir)/($binary).exe'

print $'Packaging ($binary) v($version) for ($target) in ($src)...'
print $'Executable path is ($executable)...'

if not ('Cargo.lock' | path exists) {
  cargo generate-lockfile
}

rm -rf $release_dir
mkdir $release_dir

print $'Building on Windows in ($src)...'
cargo rustc -q --bin $binary --target $target --release

#
# Release packaging
#

cd $src

print $"Release directory: ($release_dir)"
ls $release_dir | print

cp -r LICENSE* $release_dir
cp -r README* $release_dir

cd $release_dir

let artifact_filename = $'($binary)-($version)-($target).exe'

print $'(char nl)(ansi g)Build artifacts:(ansi reset)'; hr-line; ls | print
print $'Release artifact name: ($artifact_filename)'

cp $binary_filename $artifact_filename

print $'Artifact ($artifact_filename) is ready'
echo $'artifact=($artifact_filename)' | save --append $env.GITHUB_OUTPUT


# Print a horizontal line marker
def 'hr-line' [
    --blank-line(-b)
] {
    print $'(ansi g)---------------------------------------------------------------------------->(ansi reset)'
    if $blank_line { char nl }
}