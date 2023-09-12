# Brim

Brim is an optimizing brain* interpreter than nonetheless strives to be very
easy and pleasant to use.

## Features

### Details

Brim uses a wrapping 30000-cell tape, with a single wrapping signed byte in
each cell.

### Debug

If compiled in debug mode, or with the feature flag `debug`, brim will
recognize the character `;`, and will dump several relevant bits of info
(the tape, the tape pointer, and three instructions for context) to
output.

### File I/O

With the `-i | --input` and `-o | --output` flags, brim can read/write to/from
files instead of stdin/stdout.

Regardless of source or destination, all I/O is buffered to provide a fast and
reliable experience (the output is flushed on each newline, and upon exit).

### Optimizations

Brim internally uses a token-based intermediary structure to execute the code.
First, it groups together add, subtract, left, and right instructions to save
needless cycles.

Then, it performs several macro-optimizations to reduce common operations to
a single "instruction". Currently, it recognizes:

- Zeroing a cell (`[-]`)
- Setting a cell to a value (`[-]++++`)
- Soon, moving one cell to another (`[->+<]` / `[>+<-]`)

The token-based structure makes these trivial to recognize, since repeating
instructions have already been collapsed into one.

Finally, it iterates over each bracket (`[`/`]`) and pre-loads its destination
token, so executing them requires zero lookup time (it just overrides the IP).

These are all run in a buffered I/O environment, as detailed above.

## Roadmap

Brim isn't finished yet! My hopes for the future include:

- More macro-optimizations
- Heavy micro-optimization of brim itself
- More variations (non-wrapping, signed, varying tape size)
