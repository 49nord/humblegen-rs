# Rust Generated Code Guide

This guide gives an overview of the code generated by the humblegen codegen Rust backend.

**Note**: the generated Rust code is documented as well.
This document's purpose is no more than to to fit the gaps.

## Data Types

## Services

A service definition is rendered to a Rust trait with the same name.
We refer to this trait as **handler trait**.
Until async traits are stabilized, we rely on the `async_trait` proc macro.

For a given humblespec, there is also an `enum Handler` with variants named after each defined service.

A generated `Builder` struct is used to construct an HTTP server that exposes trait objects that implement one or more `handler trait`s.

### Server-Side

The usage story for a server implementation of a humblespec service is as follows:

* Define a **handler** type and implement the generated handler trait for service `$ServiceName` (remember to use `async_trait(Sync)` for the `impl` block).
* Instantiate the handler type and wrap it in an `Arc`.
* Move that `Arc` into the generated `enum Handler`'s `Handler::$ServiceName`.
* (Repeat the above for all handlers to be registered with the server)
* Instantiate a builder.
* Use `Builder::add(root, h)` to add `h: enum Handler` to the builder, rooted at URI `root: str`.
* Finish the builder and start listening by invoking `Builder::listen_and_run_forever`.




