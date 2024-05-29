# Unreleased

**Breaking changes**

* Remove `Command.pos_args()` and `key_args()`, use `consume_args()`.
* Return `ErrorKind::Other` from `run()` on command failure.

**Improvements**

* Add `Command.consume_args()` for convenient argument handling.
* Allow `@` in unquoted strings.

# 0.3.0 (2024-05-25)

**Breaking changes**

* [`44df27d`] Return `Box<dyn Error>` from `Argument.parse()`, like `Runner`.

**Improvements**

* [`eb25a24`] Add `Command.pos_args()` and `key_args()` for filtering argument types.
* [`f4bcae6`] Add `Argument.name()` to identify arguments.
* [`5ff556d`] Implement `Clone` for `Command` and `Argument`.

[`44df27d`]: https://github.com/erikgrinaker/goldenscript/commit/44df27daf3f6f31adde25238693daeb17611a057
[`eb25a24`]: https://github.com/erikgrinaker/goldenscript/commit/eb25a24136ee0f90ec0f067e169fb70114003743
[`f4bcae6`]: https://github.com/erikgrinaker/goldenscript/commit/f4bcae6f4dcd400deed1e2ad49de876ccccb6a25
[`5ff556d`]: https://github.com/erikgrinaker/goldenscript/commit/5ff556dff5875243aff5efc914689da1078f1431

# 0.2.0 (2024-05-05)

**Breaking changes**

* [`3ce4590`] Use `Box<dyn Error>` for `Runner` methods.
* [`fe62af3`] Default to "ok" for command blocks that don't yield any output.
* [`c98db05`] Mark `Command` and `Argument` as `non_exhaustive`, to allow extending them.

**Improvements**

* [`51c34d9`] Relax dependency version requirements.
* [`f911c66`] Add `generate()` to generate output for a goldenscript input.
* [`5f49b9d`] Add `Argument.parse()` to parse values into e.g. integers or booleans.
* [`cc0936f`] Add `Command.line_number` with the command's position in the script.
* [`cc0936f`] Add `Runner` error context such as the command or hook name and line number.
* [`456ae1b`] Make parse errors more concise.

[`3ce4590`]: https://github.com/erikgrinaker/goldenscript/commit/3ce4590a0794f94ee58c1fdfc647185819b6de4f
[`fe62af3`]: https://github.com/erikgrinaker/goldenscript/commit/fe62af3c3504acf4078d1f89a56be91c91d1e578
[`c98db05`]: https://github.com/erikgrinaker/goldenscript/commit/c98db054d5e940ada76dbdc855925cfc2f6e7ee8
[`51c34d9`]: https://github.com/erikgrinaker/goldenscript/commit/51c34d90a1c951d1f36b52421cf4b025bed5a5d3
[`f911c66`]: https://github.com/erikgrinaker/goldenscript/commit/f911c66312a6e9c4e6daf8ee9c5f1f810c3779c1
[`5f49b9d`]: https://github.com/erikgrinaker/goldenscript/commit/5f49b9dc7e59a3069808ededd09af06ec30338b2
[`cc0936f`]: https://github.com/erikgrinaker/goldenscript/commit/cc0936fbf0238bdbf382f1d2c8c654f4c4e25dc3
[`456ae1b`]: https://github.com/erikgrinaker/goldenscript/commit/456ae1b22f4b34eaee248bceac4dcb16e418369cc

# 0.1.0 (2024-05-01)

Initial release.