# A humble code generator

`humblegen` is an **experimental** code generator written in Rust that allows defining data structures in a custom language and generating declarations as serialization and deserialization implementations in target languages. It is similar to [protobuf](https://developers.google.com/protocol-buffers) but focuses on simple uses cases (i.e. small applications) where simplicity trumps performance. Core design goals are:

* Support Rust and Elm.
* Use JSON on the wire.
* As indistinguishable from hand-written serialization code as possible.

## Full Example Project

See `./example`.

## Usage

You can compile and install `humblegen` directly from github:

```
$ cargo install --git https://github.com/mbr/humblegen-rs
```

Check out `generator/tests/rust/showcase/spec.humble` for an overview of the format. Then just run:

```
$ humblegen -l rust -a backend myfile.humble
$ humblegen -l elm -a client myfile.humble
$ humblegen -l docs myfile.humble
```

Wherever you use the generated code, put the following into `Cargo.toml`:

```toml
[dependencies]
humblegen-rt = ...TODO:publish_to_crates.io...
serde = { version = "1.0.110", features = [ "derive" ] }
tokio = { version = "0.2.20", features = ["rt-threaded", "tcp", "macros"] }
```

### As a build dependency

`humblegen` can be used in your `build.rs` directly, this has the advantage of automatically recompiling the Rust program whenever the underlying spec changes. To enable, you should first add `humblegen` to your build dependencies:

```toml
[build-dependencies]
humblegen = "*"
```

(Using `cargo add --build` via [cargo edit](https://crates.io/crates/cargo-edit) is recommended instead)

Then, add the following line to `build.rs`:

```rust
humblegen::build("path/to/spec.humble").expect("compile humble");
```

Finally, import the module (which in this version of humblegen is always in a file called `protocol.rs`):

```rust
mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
```
