# Brain Acceptance Runner Review 3

## Verdict

APPROVED AFTER MINIMAL FIXES

## Review

- The previous egg-info snapshot contained files only. A pre-existing but empty `src/*.egg-info` directory therefore looked newly created and was deleted. Cleanup now snapshots both files and directories, preserves every pre-existing path, restores pre-existing files byte-exactly, and removes only paths that did not exist before the run.
- An egg-info directory created during the run is removed entirely, including nested files, empty directories, and symlinks. This prevents generated metadata from shadowing the installed distribution while leaving no generated scope artifacts.
The original rerun loaded the failed test back into the same interpreter after the full suite. Global state, `sys.path`, environment mutations, and in-process side effects from other tests could therefore make a real regression pass "alone." Reruns now execute in a fresh child process, from the lane root, with the environment captured before the full suite.
- Discovery import errors are classified before reruns and are never retried. `_FailedTest` IDs remain hard failures, so an import error cannot be masked by a later successful module import.
- Standard `subTest` suffixes are stripped before loading the parent test. A real failure in that parent still fails the isolated rerun; a skipped rerun is not accepted as a pass.
- The full-suite `testsRun` count, the 900-test floor, baseline matching, nonzero exit for new failures/import errors/low test count, and zero exit on clean or accepted-flake runs remain unchanged.

## Residual Risks

- A genuinely order-dependent regression can fail in the full suite but pass in the isolated rerun and be reported as flaky. This is inherent to the requested one-test-alone strategy.
- Filesystem side effects outside `build/` and `src/*.egg-info` are not reverted, so a first-run side effect can still influence the isolated rerun.
- A rerun test may repeat external side effects, such as network calls or writes outside the recognized build-artifact paths.
- Python `finally` cannot clean artifacts after a hard process kill or external termination.
- The rerun has no timeout; a hanging failed test can still hang acceptance.

## Checks

- Performed a manual static review of the complete runner and its failure classification, cleanup, and exit paths.
- Did not run the required Python `ast.parse` acceptance command, the Brain suite, or other Python/Node/npm/Cargo tests because this worker environment explicitly cannot launch those runtimes.
