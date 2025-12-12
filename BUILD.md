# Building rabbitmqadmin

## Standard Build

```bash
cargo build --release
```

## Alpine Linux / musl-based Systems

When building on Alpine Linux or other musl-based distributions, you must configure Rust to dynamically link against the system's musl libc to avoid segmentation faults.

### The Problem

The `x86_64-unknown-linux-musl` target statically links musl by default. When your binary dynamically links to Alpine's OpenSSL (which itself links to Alpine's musl), you end up with two different musl instances in the same process, causing segmentation faults at runtime.

### The Solution

Set the `RUSTFLAGS` environment variable to dynamically link musl:

```bash
RUSTFLAGS="-C target-feature=-crt-static" cargo build --release
```

### Alternative: Static Linking

Alternatively, you can statically link OpenSSL by enabling the `vendored` feature in dependencies that use OpenSSL. However, this approach is not currently configured in this project.

### References

- [Rust Users Forum: SIGSEGV with program linked against OpenSSL in an Alpine container](https://users.rust-lang.org/t/sigsegv-with-program-linked-against-openssl-in-an-alpine-container/52172)
- [GitHub Discussion #99](https://github.com/rabbitmq/rabbitmqadmin-ng/discussions/99)
