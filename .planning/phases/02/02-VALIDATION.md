# Phase 2 Validation: Content Types and Output Modes

## Purpose

Provide a single traceability artifact that maps Phase 2 requirements and context decisions to split plans and executable verification commands.

## Plan Coverage Matrix

| Requirement | Plan         | Verification                                                  |
| ----------- | ------------ | ------------------------------------------------------------- |
| HTTP-04     | 02-01        | `cargo test --test fetch_request_options_integration`         |
| HTTP-05     | 02-01        | `cargo test --test fetch_request_options_integration`         |
| HTTP-06     | 02-01        | `cargo test --test fetch_request_options_integration`         |
| OUT-02      | 02-01, 02-03 | `cargo test --test fetch_output_modes_integration`            |
| OUT-03      | 02-01, 02-03 | `cargo test --test fetch_output_modes_integration`            |
| CONT-01     | 02-02        | `cargo test --test fetch_envelope_integration -- --nocapture` |
| CONT-02     | 02-02        | `cargo test fetch::handlers:: -- --nocapture`                 |
| CONT-03     | 02-02        | `cargo test fetch::handlers:: -- --nocapture`                 |
| CONT-04     | 02-02        | `cargo test fetch::handlers:: -- --nocapture`                 |
| CONT-05     | 02-02        | `cargo test fetch::handlers:: -- --nocapture`                 |
| EXTR-05     | 02-03        | `cargo test --test fetch_envelope_integration`                |
| OUT-04      | 02-03        | `cargo test --test fetch_file_output_parity_integration`      |

## Checker Finding Closure

| Checker finding                    | Closure artifact                                                      |
| ---------------------------------- | --------------------------------------------------------------------- |
| Single overloaded plan             | Split into `02-01`, `02-02`, `02-03` via `02-PLAN.md` index           |
| 20 files in one plan               | Each plan constrained to 6-7 files in `files_modified`                |
| Research open questions unresolved | Resolved in `02-RESEARCH.md` patch section                            |
| Missing validation artifact        | This file (`02-VALIDATION.md`)                                        |
| OUT-04 parity missing              | `02-03-PLAN.md` explicit parity tests for default/no-frontmatter/json |
| D2-24 not executable               | `02-03-PLAN.md` includes state-alignment checkpoint task              |

## D2 Decision Traceability

| Decision                                        | Implemented by |
| ----------------------------------------------- | -------------- |
| D2-05, D2-06, D2-07, D2-08                      | Plan 02-01     |
| D2-01, D2-02, D2-09, D2-10, D2-11, D2-12, D2-13 | Plan 02-02     |
| D2-14, D2-15, D2-16, D2-17, D2-18, D2-24        | Plan 02-03     |

## Phase Exit Commands

- `cargo test --test fetch_request_options_integration`
- `cargo test --test fetch_envelope_integration`
- `cargo test --test fetch_output_modes_integration`
- `cargo test --test fetch_file_output_parity_integration`
- `cargo test`
- `cargo clippy -- -D warnings`

## Exit Criteria

- All matrix rows pass on CI.
- OUT-04 parity tests pass in all output modes.
- `.planning/STATE.md` reflects the split plan execution state for Phase 2.
