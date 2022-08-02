# eSDR

A Software Defined Radio app written in Rust with [FutureSDR](https://www.futuresdr.org/), inspired by GNU Radio.

## Adding new blocks

The blocks bundled with eSDR right now aren't ideal. You can add more
blocks by registering them under `crate::blocks::ESDRBlockType` and
following one of the examples under `src/blocks/`.

## TODO

* Introduce saner, more reusable, DSP blocks
* Add waterfall block
* Add block for other charts (time series? something else?)

* Make more fields updatable (right now only SoapySDR supports messages
  for updating fields)
* Add support for post-processing scalars from the UI (e.g. adding offset)

* Improve how frequency is input (90.9 MHz instead of 90900000)
* Make UI read-only when radio is running (for non-updatable fields)

* Allow for saving/opening graphs
* Allow for copy/pasting blocks
* Hotkeys for adding blocks, save, open, copy, paste, etc

* Improve error handling (right now it's a bit of a mess, lots of unwraps,
  not handling errors while initializing radio with bad flowgraph state, etc)
* Some test coverage
* CI setup
