# Railroad Runners Wrapper

This is a wrapper around the game that runs it in real time. You need `mipsy` installed for it to work.

## Usage

```sh
cargo run -- path/to/railroad-runners.s --mipsy-path path/to/mipsy/executable
```

On UNSW CSE, you can omit the `--mipsy-path` flag since the default path should be correct. If not, try `--mipsy-path $(1521 which mipsy)`.

Use the `--help` flag for more info.
