# Fuzzing `gfx-memory`

First, install [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) by running
```sh
$ cargo install cargo-fuzz
```

List available fuzz targets with
```sh
$ cargo fuzz list
```

Then run
```sh
$ cargo fuzz run <fuzz target>
```
to start a fuzzing run.
