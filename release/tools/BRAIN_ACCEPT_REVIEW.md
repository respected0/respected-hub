# Brain Acceptance Runner Review

## Verdict

APPROVED

## What was checked

- Baseline matching uses exact test IDs, one per line, so IDs containing spaces are preserved.
- Both test failures and test errors are considered failures; the runner no longer treats them differently.
- SubTest IDs are preserved as exact strings, so a new failing subTest of a baseline test is not silently masked by the parent test ID.
- Discovery import errors are now explicitly forced to a non-zero exit, even if their ID somehow matched the baseline.
- A hard minimum of 900 discovered/executed tests is enforced, so a zero-test or truncated run cannot pass.
- `PYTHONPATH` is exported with an absolute `src` path and the platform-specific path separator, preserving subprocess import behavior on Windows.

## Defects fixed

- Zero or fewer than 900 tests could previously pass if no new baseline-mismatched failure was reported.
- Discovery import errors were only rejected indirectly if their ID was outside the baseline; they are now always rejected.

## Residual risks

- The gate compares failure IDs, not failure causes. A baseline test that starts failing for a new reason with the same ID can still be masked.
- Tests that are silently skipped or not discovered because of a naming or suite-structure change may evade the baseline mismatch, though the 900-test floor and explicit import-error check reduce that risk.
- Environment-dependent flakiness outside the recorded baseline can still cause the lane to reject or, if the flaky ID is baselined, be masked.