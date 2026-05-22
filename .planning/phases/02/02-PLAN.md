---
plan: 02
wave: split-index
depends_on:
  - 01
files_modified:
  - .planning/phases/02/02-PLAN.md
  - .planning/phases/02/02-01-PLAN.md
  - .planning/phases/02/02-02-PLAN.md
  - .planning/phases/02/02-03-PLAN.md
  - .planning/phases/02/02-VALIDATION.md
requirements_addressed:
  - CONT-01
  - CONT-02
  - CONT-03
  - CONT-04
  - CONT-05
  - OUT-02
  - OUT-03
  - OUT-04
  - HTTP-04
  - HTTP-05
  - HTTP-06
  - EXTR-05
autonomous: true
---

# Phase 2 Plan Index: Content Types and Output Modes

## Objective

Replace the overloaded single Phase 2 plan with a sequenced 3-plan execution strategy that keeps each plan within safe scope, resolves open decisions, and adds explicit validation traceability.

## Split Strategy

1. `02-01-PLAN.md` (Wave 1)
   - Request surface + typed output mode selection.
   - HTTP-04/05/06 and CLI mode validation foundation.

1. `02-02-PLAN.md` (Wave 2, depends on `02-01`)
   - Content-type router and handler matrix.
   - CONT-01..05 behavior implementation.

1. `02-03-PLAN.md` (Wave 3, depends on `02-02`)
   - Envelope/output finalization.
   - EXTR-05 and OUT-02/03/04 completion.
   - Explicit OUT-04 parity assertions and D2-24 state-alignment checkpoint.

## Wave and Dependency Model

- Wave 1: `02-01`
- Wave 2: `02-02` -> requires `02-01`
- Wave 3: `02-03` -> requires `02-02`

No same-wave file overlap is allowed; each plan owns a distinct file set to avoid execution conflicts.

## Verification Commands

- `cargo test --test fetch_request_options_integration`
- `cargo test --test fetch_envelope_integration`
- `cargo test --test fetch_output_modes_integration`
- `cargo test --test fetch_file_output_parity_integration`
- `cargo test`
- `cargo clippy -- -D warnings`

## Acceptance Criteria

- Each plan has <= 8 files in `files_modified` and explicit `depends_on` sequencing.
- Research open questions are resolved and documented in `02-RESEARCH.md`.
- Validation traceability is captured in `02-VALIDATION.md`.
- Final plan includes explicit OUT-04 byte-parity tests and D2-24 alignment checkpoint.

## Plan Artifacts

- `02-01-PLAN.md`
- `02-02-PLAN.md`
- `02-03-PLAN.md`
- `02-VALIDATION.md`
