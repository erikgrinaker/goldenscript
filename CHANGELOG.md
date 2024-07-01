# 0.7.0 (2024-07-01)

**Improvements**

* [`29903fa`]: Add `>` syntax to use the raw line as the command name.
* [`5d2519c`]: Allow tags before command.

[`29903fa`]: https://github.com/erikgrinaker/goldenscript/commit/29903faed2813329a89b5d79080ec6843c1bac2d
[`5d2519c`]: https://github.com/erikgrinaker/goldenscript/commit/5d2519c2623765bb72b6648334763eba09a0f9e7

# 0.6.0 (2024-06-13)

**Improvements**

* [`2bd0058`] Add `[]` syntax for command tags, exposed as `Command.tags`.
* [`686e261`] Add `\x` escape sequence for hex bytes.
* [`0b62d19`] Add `\u{}` escape sequence for Unicode characters.
* [`a096929`] Allow empty commands, keys, and prefixes.

[`2bd0058`]: https://github.com/erikgrinaker/goldenscript/commit/2bd0058886111487472012d249184ce9663f1299
[`686e261`]: https://github.com/erikgrinaker/goldenscript/commit/686e26168e901995f1311dad2a51345cee9ac9b2
[`0b62d19`]: https://github.com/erikgrinaker/goldenscript/commit/0b62d19e48b14046d18c3796e6cd3c253ba53bb4
[`a096929`]: https://github.com/erikgrinaker/goldenscript/commit/a096929774d44cbb979add9853b7eb45493ce2f0

# 0.5.0 (2024-05-31)

**Bug Fixes**

* [`9a824ad`] Fix spurious prefix emission with blank lines or empty output.

**Improvements**

* [`4fad99e`] Add `Runner.start_command()` and `end_command()` hooks.

[`9a824ad`]: https://github.com/erikgrinaker/goldenscript/commit/9a824add3e26c3e1ba31611f9f962a734700a5b3
[`4fad99e`]: https://github.com/erikgrinaker/goldenscript/commit/4fad99e7f8c5fb604da35ed54b9037f2d1058d59

# 0.4.0 (2024-05-29)

**Breaking changes**

* [`92ca419`] Remove `Command.pos_args()` and `key_args()`, use `consume_args()`.
* [`1b54d07`] Return `ErrorKind::Other` from `run()` on command failure.

**Improvements**

* [`bc9c253`] Add `!` syntax to expect command failures (panics or errors).
* [`ae78f9e`] Add `Command.consume_args()` for convenient argument handling.
* [`6e8c185`] Allow `@` in unquoted strings.

[`92ca419`]: https://github.com/erikgrinaker/goldenscript/commit/92ca419d7618419adc4890994f40e1a577c705f4
[`1b54d07`]: https://github.com/erikgrinaker/goldenscript/commit/1b54d07d47a379b6bc4c8b95f31d7b06c79394ff
[`bc9c253`]: https://github.com/erikgrinaker/goldenscript/commit/bc9c2539c144fecc7496017113b1d7759c1a4794
[`ae78f9e`]: https://github.com/erikgrinaker/goldenscript/commit/ae78f9eef1b5fc8007bd63165c4e8493e93ec692
[`6e8c185`]: https://github.com/erikgrinaker/goldenscript/commit/6e8c185a252045100a99782317730e6ed2de05c3

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