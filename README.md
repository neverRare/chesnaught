# Chesnaught

Toy Chess and [Chess960] engine that speaks [UCI].

## Installation

This is a toy project that I don't feel like providing prebuilt executables. Compile it yourself using [Cargo], remember to put `--release`.

Chesnaught is a command line software. You'll need a GUI in order to use Chesnaught. Here are my recommendation:

- [En Croissant] &ndash; User friendly option
- [Cute Chess] &ndash; Lightweight option

## Playing Motif

(It's currently in development, it may change)

## Options

Before playing, consider configuring it first to give it more computation advantage.

### Thread

The amount of threads that Chesnaught will use during search. Optimal setting: the same number as your CPU cores. You can provide less if you want. Giving more may not help.

Chesnaught actual uses more than the alloted threads but the extra threads should be lowly prioritized by the OS e.g. they're blocked most of the time. Here are the detailed list of threads used:

- IO thread &ndash; Processes inputs, sends output, and sends instruction to the engine thread. It is important that the IO thread and the engine thread are separate so the IO thread can process inputs while the engine is thinking. Should only be awake briefly whenever there is an input.
- Engine thread &ndash; Performs analysis, may spawn multithreaded analysis threads but this thread will block and wait for them to finish.
- Multithreaded analysis threads &ndash; Has the same number as the option. Spawned by the engine thread.
- Timing thread &ndash; Used for timings, it's only purpose is to sleep for a set time and then tell the engine to stop.
- Search tree garbage collector &ndash; The only real bottleneck outside of the allocated analysis threads. Because the trees are huge and complex, it can take a while to free them. Chesnaught simply sends them to this thread in order to be freed asynchronously. Garbages are queued. This could be a bottleneck but since creation of large trees requires time, which would allocate time freeing garbages, this shouldn't be a problem. Garbage collection should also be fairly brief and therefore not affect the analysis threads.

### Hash

The amount of memory in MiB that Chesnaught will use for storing previously computed board position to avoid duplicate computation. Chesnaught will not allocate it right away but it'll start with none then it'll grow as needed. Chesnaught could limit itself to a lower setting but it is guaranteed to never exceed provided limit. Optimal setting: as high as you're willing to give, although 1024 Mib (1 GiB) should be more than enough.

## Playing

If you want to play against Chesnaught. You'll need to put a limit as otherwise it'll not play as it searches forever. You can limit it by:

- Time (Recommended)
- Depth &ndash; The number of plies it'll search
- Nodes &ndash; The number of positions it'll search. (Poorly supported, it'll overcount)

## Analysis

You can perform analysis with Chesnaught to see what it thinks. It shouldn't be used for analyzing games, use [Stockfish] instead. Chesnaught doesn't provide multiple lines as it's mainly designed for playing.

Chesnaught does provide centipawn analysis but this is an approximation. Chesnaught's actual score is a compound number that is hard to condense into a single number.

[Chess960]: https://en.wikipedia.org/wiki/Chess960
[UCI]: https://en.wikipedia.org/wiki/Universal_Chess_Interface
[Cargo]: https://rust-lang.org/
[En Croissant]: https://encroissant.org/
[Cute Chess]: https://cutechess.com/
[Stockfish]: https://stockfishchess.org/
