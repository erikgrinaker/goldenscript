# Unreleased

Backwards-incompatible changes:

* Default to "ok" for command blocks that don't yield any output.
* Mark `Command` and `Argument` as `non_exhaustive`, to allow extending them.

Improvements:

* Add `generate()` to generate output for a goldenscript input.
* Add `Argument.parse()` to parse values into e.g. integers or booleans.
* Add `Command.line_number` with the command's position in the script.
* Add `Runner` error context such as the command or hook name and line number.
* Make parse errors more concise.

Bug fixes:

* Relax dependency version requirements.