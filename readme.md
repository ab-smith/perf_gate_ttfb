# TTFB Performance Gate 

Simple CLI tool to measure the TTFB (Time To First Byte) for a specific URL and decide on a gate for it with a threshold based on 95th percentile.

The CLI has an inlined documentation with -h

A compiled version is also available. You might need `chmod +x` after the download.

You need Rust toolchain installed. After Clone, use cargo:

```
$ cargo run -- -h
```

```
$ cargo build --release
```