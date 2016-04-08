# `rs-transducers`

[![Build Status](https://travis-ci.org/benashford/rs-transducers.svg?branch=master)](https://travis-ci.org/benashford/rs-transducers)

An attempt at implementing Clojure style transducers in Rust.

## What is a transducer?

When first introduced into Clojure, the concept of transducers caused a [lot of confusion](https://news.ycombinator.com/item?id=8143905).  The best overview is part of the [Clojure reference](http://clojure.org/reference/transducers).

Essentially a transducer separates the application of functions on data from the structure of the data.  For example the higher-order functions like `map` can be expressed in such a way that could be applied to a vector, but also an iterator, but also a channel containing data passed between threads.

This package contains a somewhat simplified implementation of Clojure's transducer implementation.

## Transducers

### Provided transducers

### Applying transducers

## Applications

### Provided applications

### Implementing applications

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
