# `rs-transducers`

[![Build Status](https://travis-ci.org/benashford/rs-transducers.svg?branch=master)](https://travis-ci.org/benashford/rs-transducers)
[![](http://meritbadge.herokuapp.com/rs_transducers)](https://crates.io/crates/rs_transducers)

An attempt at implementing Clojure style transducers in Rust.

## What is a transducer?

When first introduced into Clojure, the concept of transducers caused a [lot of confusion](https://news.ycombinator.com/item?id=8143905).  The best overview is part of the [Clojure reference](http://clojure.org/reference/transducers).

Essentially a transducer separates the application of functions on data from the structure of the data.  For example the higher-order functions like `map` can be expressed in such a way that could be applied to a vector, but also an iterator, but also a channel containing data passed between threads.

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
use rs_transducers::applications::vec::Into;

let source = vec![1, 2, 3, 4, 5];
let transducer = transducers::filter(|x| x % 2 == 0);
println!(source.transduce_into(transducer));
```

This will print: `[2, 4]`.

Transducers can be composed, so complex map/filter/etc. operations can be expressed simply.

```rust
let transducer = rs_transducers::compose(transducers::drop(5),
                                         transducers::filter(|x| x % 2 == 0));
```

### Provided transducers

`map` - takes a function of type `Fn(I) -> O` and returns a `MapTransducer` that implements `Transducer<I, O>`.  Also `map_indexed` which takes a function of type `Fn(usize, I) -> O`.

`mapcat` - takes a function of type `Fn(I) -> OI` where `OI` implementes `IntoIterator<Item=O>` and returns a `MapcatTransducer` that implements `Transducer<I, O>`.

`filter` and `remove` - takes a function of type `Fn(I) -> bool` and returns a `FilterTransducer` that implements a `Transducer<I, I>`.  `filter` will retain those that match the condition, `remove` is the opposite.

`keep` - takes a function of type `Fn(I) -> Option<O>` and returns a `KeepTransducer` that produces all `O`.  Also `keep_indexed` which takes a function of type `Fn(usize, I) -> Option<O>`.

`partition` and `partition_all` - takes a `usize` determining the size of each partition and returns a `PartitionTransducer` that implements `Transducer<I, Vec<I>>`.  The difference between the two is that `partition_all` will return the final partition incomplete, where `partition` will not.  Also `partition_by` that groups data together as long as the provided function returns the same value.

`take` and `drop` - takes a `usize` and return a transducer that implements `Transducer<I, I>` that takes or drops the appropriate number of elements.

`take_while` and `drop_while` - take or drop values while the predicate remains true.

`replace` - takes a `HashMap<T, T>` (where `T` must implement `Clone`) and returns a `ReplaceTransducer` which will replace each instance of a given key with a clone of the corresponding value.

`interpose` - takes a cloneable value `T` and returns a transducer which, when applied, interposes that value with each value that goes through the reducing function.

TODO - other transducers, at a minimum implement all those that Clojure does.  Specifically TODO are: `dedupe`, `random-sample`.

### Implementing transducers

The initial version of this library attempted to simpify what a transducer was by trying to factor out the need for a "reducing function" (please see the Clojure documentation for definition of these terms).  By not having such a function then we didn't really have a transducer, just something that could be used for similar ends.  But it soon became apparent that both reducing functions and transducers will be needed; the reason for this is it is the only way certain transducers (e.g. `mapcat`) can be applied to certain things (e.g. channels).

Implementing a transducer, therefore requires implementing two traits `Transducer` for the transducer itself and `Reducing` for the reducing function returned by the transducer when applied to the previous reducing function.

#### `Transducer`

TBC

#### `Reducing`

TBC

## Applications

Transducers need to be applied to a source of data to have an effect.  The initial example used the `Into` trait to add `transduce_into` to vectors; as the name suggests, this is analogous to `into_iter()` in that it consumes the original data, applies the transducer and returns a new vector.

### Provided applications

Implemented so far are transducer applications for:

#### `Vec<T>`

This comes in two forms `Into` that adds a `transduce_into` to vectors, this consumes the original vector; and the `Ref` trait that adds `transduce_ref` to vectors, this leaves the original vector unchanged and returns a new one based on feeding references to the source data through the transducer.

#### `Iterator`

The trait `TransduceIter` adds a `transduce` to iterators which returns a new iterator.

#### Channels

Unlike operations solely defined on iterators, transducers can be applied to any sequence of data, including streams of data through channels between threads.

One compromise is necessary since Rust's channels are concrete `Sender` and `Receiver` types, not implementing any traits, we cannot implement one of these channels (not without creating two pairs of channels, but that would need an additional thread to pipe messages between them).  Instead we wrap the `Sender` type with a new `TransducingSender`. 

For example (from the tests):

```rust
let transducer = super::compose(transducers::partition_all(6),
                                transducers::filter(|x| x % 2 == 0));
let (mut tx, rx) = transducing_channel(transducer);
thread::spawn(move|| {
    for i in 0..10 {
        tx.send(i).unwrap();
    }
    tx.close().unwrap();
});
assert_eq!(vec![0, 2, 4, 6, 8], rx.recv().unwrap());
```

### Implementing applications

Any custom data-structure/channel/sequence/etc. can apply a transducer.

In order to do this an implementation of `Reducing` needs to be provided, to build the required data-structure.  Then:

1. By passing this to the `new` function of a transducer a new reducing function is returned.
2. Call `init` on the reducing function.
3. For each piece of data call `step`.
4. Finally call `complete`.

It is the responsibility of the implementation to retain access to the constructed data structure.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
