#!/usr/bin/env nu

let version = $env.VERSION? | default ""
if $version == "" {
    print "ERROR: VERSION environment variable is not set"
    exit 1
}

let header = $'## v($version)'

let lines = open CHANGELOG.md | lines
let start_idx = $lines | enumerate | where { |it| $it.item | str starts-with $header } | get -i 0.index

if $start_idx == null {
    print $"ERROR: Version header '($header)' not found in CHANGELOG.md"
    exit 1
}

let remaining = $lines | skip ($start_idx + 1)
let end_offset = $remaining | enumerate | where { |it| $it.item | str starts-with '## v' } | get -i 0.index | default ($remaining | length)

let section = $lines | skip $start_idx | take ($end_offset + 1)

$section | str join "\n" | save --force release_notes.md
