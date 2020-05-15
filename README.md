# A humble code generator

`humblegen` is an **experimental** code generator written in Rust that allows defining data structures in a custom language and generating declarations as serialization and deserialization implementations in target languages. It is similar to [protobuf](https://developers.google.com/protocol-buffers) but focuses on simple uses cases (i.e. small applications) where simplicity trumps performance. Core design goals are:

* Support Rust and Elm.
* Use JSON on the wire.
* As indistinguishable from hand-written serialization code as possible.
* No runtime library required for the generated code.

## Usage

You can compile and install `humblegen` directly from github:

```
$ cargo install --git https://github.com/mbr/humblegen-rs
```

Check out `tests/rust/showcase/spec.humble` for an overview of the format. Then just run:

```
$ humblegen -l rust myfile.humble
```

### As a build dependency

`humblegen` can be used in your `build.rs` directly, this has the advantage of automatically recompiling the Rust program whenever the underlying spec changes. To enable, you should first add `humblegen` to your build dependencies:

```toml
[build-dependencies]
humblegen = "*"
```

(Using `cargo add --build` via [cargo edit](https://crates.io/crates/cargo-edit) is recommended instead)

The, add the following line to `build.rs`:

```rust
humblegen::build("path/to/spec.humble").expect("compile humble");
```

Finally, import the module (which in this version of humblegen is always in a file called `protocol.rs`):

```rust
mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
```