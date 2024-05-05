# Unreleased

Backwards-incompatible changes:

* Mark `Command` and `Argument` as `non_exhaustive`, to allow extending them
  in the future.

Improvements:

* Add `Argument.parse()` convenience method to parse argument values into e.g.
  integers or booleans.
* Make parse errors more concise.

Bug fixes:

* Relax dependency version requirements (was tilde requirements).