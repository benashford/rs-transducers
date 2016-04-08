# `rs-transducers`

[![Build Status](https://travis-ci.org/benashford/rs-transducers.svg?branch=master)](https://travis-ci.org/benashford/rs-transducers)

An attempt at implementing Clojure style transducers in Rust.

## What is a transducer?

When first introduced into Clojure, the concept of transducers caused a [lot of confusion](https://news.ycombinator.com/item?id=8143905).  The best overview is part of the [Clojure reference](http://clojure.org/reference/transducers).

Essentially a transducer separates the application of functions on data from the structure of the data.  For example the higher-order functions like `map` can be expressed in such a way that could be applied to a vector, but also an iterator, but also a channel containing data passed between threads.

This package contains a somewhat simplified implementation of Clojure's transducer implementation intended to be idiomatic Rust while providing the same functionality.

This library contains two parts:

1. A collection of frequently occurring transducers.
2. Implementation of applications of those transducers.

In both cases these collections can be extended.  Custom transducers can be defined, and transducers can be applied to any custom data structure or stream.

WARNING: as a result of the simplification, there is potentially some confused terminology.  At this early stage of development, I'm happy to correct these even if it involves renaming significant parts of the library.

## Transducers

An example of a transducer to filter odd numbers:

```rust
extern crate rs_transducers;

use rs_transducers::transducers;
use rs_transducers::applications::vec::Drain;

let source = vec![1, 2, 3, 4, 5];
let transducer = transducers::filter(|x| x % 2 == 0);
println!(source.transduce_drain(transducer));
```

This will print: `[2, 4]`.

Transducers can be composed, so complex map/filter/etc. operations can be expressed simply.

```rust
let transducer = rs_transducers::compose(transducers::drop(5),
                                         transducers::filter(|x| x % 2 == 0));
```

### Provided transducers

`map` - takes a function of type `Fn(I) -> O` and returns a `MapTransducer` that implements `Transducer<I, O>`.

`filter` - takes a function of type `Fn(I) -> bool` and returns a `FilterTransducer` that implements a `Transducer<I, I>`.

`partition` and `partition_all` - takes a `usize` determining the size of each partition and returns a `PartitionTransducer` that implements `Transducer<I, Vec<I>>`.  The difference between the two is that `partition_all` will return the final partition incomplete, where `partition` will not.

`take` and `drop` - takes a `usize` and return a transducer that implements `Transducer<I, I>` that takes or drops the appropriate number of elements.

TODO - other transducers, at a minimum implement all those that Clojure does.  Specifically TODO are: `mapcat`, `remove`, `take-while`, `drop-while`, `take-nth`, `replace`, `partition-by`, `keep`, `keep-indexed`, `map-indexed`, `interpose`, `dedupe`, `random-sample`.

### Implementing transducers

Custom transducers can be implemented easily by implementing the `Transducer<I, O>` trait.  Implementations must provide an `fn accept(&mut self, value: Option<I>) -> TransductionResult<O>`.

`accept` is called by the code applying the transducer to data, and is called for each element: `transducer.accept(Some(value))`.  When there is no more data, it will continue to call `transducer.accept(None)` until the transducer signals it is finished, this is because some transducers (like `partition_all` have state which is flushed at the end).

`accept` returns a `TransductionResult<O>` which is an enum with three options:

1. `End` to indicate that everything is finished and no more data should be passed through the transducer.  However, under certain circumstances, `accept` might continue to be called, so the transducer should continue to return `End`.  This option is used by transducers such as `take` to stop the process ahead of time.  `End` should also be used to acknowledge a call to `accept` with `None`.

2. `None` to indicate that this call did not produce a value, e.g. a call to a `filter` transducer that is filtering something out.  Further calls to `accept` should be made until the end of the process.

3. `Some(O)` the result of a call.

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
