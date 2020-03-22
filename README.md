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

Check out [the sample file](sample.humble) for an overview of the format. Then just run:

```
$ humblegen -l rust myfile.humble
```