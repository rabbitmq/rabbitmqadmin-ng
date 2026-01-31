#!/usr/bin/env nu

# Validates that NEXT_RELEASE_VERSION matches the git tag

let expected = $env.NEXT_RELEASE_VERSION? | default ""
let tag = $env.GITHUB_REF_NAME? | default ""

if $expected == "" {
    print "ERROR: NEXT_RELEASE_VERSION variable is not set"
    print ""
    print "Set it at: Settings > Secrets and variables > Actions > Variables"
    exit 1
}

if $tag == "" {
    print "ERROR: GITHUB_REF_NAME is not available"
    exit 1
}

# Check if this looks like a version tag
if not ($tag | str starts-with 'v') {
    print $"ERROR: This workflow should be triggered by a version tag \(v*.*.*\), not '($tag)'"
    print ""
    print "Push a tag like: git tag v($expected) && git push origin v($expected)"
    exit 1
}

# Extract version from tag (v0.14.0 -> 0.14.0)
let tag_version = $tag | str substring 1..

if $expected == $tag_version {
    print $"Version validated: ($expected) matches tag ($tag)"
} else {
    print $"ERROR: NEXT_RELEASE_VERSION \(($expected)\) does not match tag \(($tag)\)"
    print ""
    print "Either:"
    print $"  1. Update NEXT_RELEASE_VERSION to '($tag_version)' at: Settings > Secrets and variables > Actions > Variables"
    print $"  2. Or push the correct tag: git tag v($expected) && git push origin v($expected)"
    exit 1
}
