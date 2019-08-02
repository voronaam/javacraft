JVM byte code parser for FreeMiner/Minetest and Music
===
This is a playground project that seeks to extract some very basic information about java program and write a Freeminer/Minetest map out of it.

This can be used as a way to bring to live and interactive visualization of any JVM (Java, Scala, etc) code.

Current version outputs an SQL script that could be executed against an existing sqlite3 database defining the map. Leveldb is not supported (yet).

MELO mode
===

With `--melo` parameter it will generate a [MELO](https://github.com/mistodon/melo) sheet of music based on the code. Not sure why you would need it, but it was fun to write.

Dependencies
===

Rust (https://www.rust-lang.org/learn/get-started)
`sqlite` (libsqlite3-dev on Debian)

Sample usage:
===
  cargo run path/to/soure.class path/to/other.jar

    This will print out static analysis results to stdout but will not attempt to generate a map.

  cargo run -- --map=/path/to/map.sqlite path/to/soure.class path/to/other.jar

    This will print out static analysis results to stdout and will create a codecity in the supplied map.
