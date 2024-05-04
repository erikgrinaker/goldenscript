# Unreleased

Backwards-incompatible changes:

* Mark `Command` and `Argument` as `non_exhaustive`, to allow extending them
  in the future.

New features:

* Add `Argument.parse()` convenience method to parse argument values into e.g.
  integers or booleans.

Bug fixes:

* Relax dependency version requirements (was tilde requirements).