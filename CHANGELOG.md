v0.3.0 (in development)
-----------------------
- Renamed `--no-has-github` to `--no-github`
- Added `--has-stash` and `--no-stash` filter options
- Added `--language` filter option
- Added `rsclean` command
- `--root` can now be specified multiple times
- Added a short `-R` form for `--root`

v0.2.0 (2025-01-06)
-------------------
- Eliminate `--show-failures` in favor of having `--keep-going` include its
  behavior
- `--keep-going` and `--quiet` are now global options
- Merge `script` into `run`
- `list --json` now includes projects' GitHub repositories
- `pull` and `push` now skip projects without GitHub remotes
- Added `--has-github` and `--no-has-github` filter options
- Added `run-pr` command

v0.1.0 (2024-12-12)
-------------------
Initial release
