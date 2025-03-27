# bunnies

A fast chess library, mainly targeting chess engines. **Currently in alpha**.

## General features
- Game state representation
- Move generation and application (and un-application)
- FEN parsing and generation
- UCI generation
- Full PGN parsing and generation
  - Support for comments, NAGs, and variations
  - Support for nested variations
- Perft testing
- Game termination detection (and draw detection)

## Optimizations
`bunnies` makes use of the following optimizations:
- Bitboards for board representation
- Pseudo-legal move generation
- Make-unmake move application (eliminating the need to copy board state)
- Magic bitboards for sliding piece attack generation
- Precomputed attacks for knights and kings
- Zobrist hashing for board state hashing

`bunnies` is currently able to generate ~15M moves/thread/second (as measured on an M1 Macbook Pro),
which is not nearly as fast as programs like `Stockfish` (which is able to generate
~120M moves/thread/sec under the same conditions).

## Contribution
`bunnies` needs the following to make 1.0.0 happen:
- Some more benchmarks
- Some more integration tests
- Is the public API good enough?
- More docs

Feedback and contributions welcome!