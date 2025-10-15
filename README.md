# Visitor Pattern in Rust

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/visitor.svg
[crates-url]: https://crates.io/crates/visitor
[docs-badge]: https://docs.rs/visitor/badge.svg
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[docs-url]: https://docs.rs/visitor
[license-badge]: https://img.shields.io/crates/l/visitor
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/visitor/workflows/CI/badge.svg
[actions-url]:https://github.com/fast/visitor/actions?query=workflow%3ACI

## Overview

This crate provides traits and proc macros to implement the visitor pattern for arbitrary data structures. This pattern is particularly useful when dealing with complex nested data structures, abstract trees and hierarchies of all kinds.

## Minimum Rust version policy

This crate is built against the latest stable release, and its minimum supported rustc version is 1.85.0.

The policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if Visitor 1.0 requires Rust 1.60.0, then Visitor 1.0.z for all values of z will also require Rust 1.60.0 or newer. However, Visitor 1.y for y > 0 may require a newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
