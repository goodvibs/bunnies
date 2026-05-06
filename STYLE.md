# Naming and Terminology Conventions

This document defines naming conventions for `bunnies` and `bunnies-pgn`.

## Primary Endpoint Terms

- Prefer `from` and `to` for move endpoints.
- If a function takes both, keep them adjacent and ordered as `(from, to)`.
- Avoid mixing `src/dst` and `from/to` in the same scope.

## Verbosity and Clarity

- Prefer names that accurately describe the value, even if somewhat verbose.
- Keep verbosity within reason, but prioritize correctness and readability over brevity.
- Use short names only in tiny, obvious scopes (for example loop counters).

## Abbreviations

- Prefer long-form semantic names in broad scopes:
  - `move` instead of `mv`
  - `position` instead of `pos`
  - `opponent` instead of `opp`
- Abbreviations are acceptable in local, performance-oriented contexts when intent is still clear.

## Bitboard Naming

- `Bitboard` remains the type name.
- Variables and parameters of this type should use a `*_mask` suffix.
- Prefer `*_mask` over `*_bb`.
- Avoid using both forms for the same concept in one scope.

## Square Naming

- Do not require type-in-name by default.
- Prefer semantic names like `from`, `to`, `king`, `attacker`.
- Add `_square` only when disambiguation is needed in the same scope.

## Side-to-Move Terms

- Keep `STM` for const generic side-to-move.
- Prefer `side_to_move` and `opponent` for runtime values.

## Boolean Naming

- Use `is_`, `has_`, or `can_` prefixes for booleans.

## Docs and Comments

- Use one term per concept consistently.
- Prefer `from/to` and `mask` wording in docs.
