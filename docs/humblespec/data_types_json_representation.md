# Data Types JSON Representation

This document describes how humblespec data types are represented as JSON.

Empty, which is the unit type, is represented using `null`. A decoder or encoder
MAY ignore the actual value transmitted since the result of an encoding or
decoding operation is statically known.
