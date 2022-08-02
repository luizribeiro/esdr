# eSDR

A Software Defined Radio app written in Rust with [FortuneSDR](https://www.fortunesdr.org/), inspired by GNU Radio.

## TODO

* Cleanup code a bunch so it's easier to contribute
* Document how to add new blocks
* Add screenshot to README
* Improve error handling (right now it's a bit of a mess, lots of unwraps,
  not handling errors while initializing radio with bad flowgraph state, etc)
* Improve how frequency is input (90.9 MHz instead of 90900000)
* Add support for post-processing scalars from the UI (e.g. adding offset)
* Make UI read-only when radio is running (for non-updatable fields)
* Make more fields updatable (right now only SoapySDR supports messages
  for updating fields)
* Introduce saner, more reusable, DSP blocks
* Add waterfall block
* Add block for other charts (time series? something else?)
* Allow for saving/opening graphs
* Allow for copy/pasting blocks
* Hotkeys for adding blocks, save, open, copy, paste, etc
* CI setup
* Some test coverage
