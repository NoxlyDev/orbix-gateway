# HSDEV

`orbixdev` is a simple diesel Postgres migration tool. It is designed to simply running a Postgres database migration with diesel.

## Installing orbixdev
`orbixdev` can be installed using `cargo`
```shell
cargo install --force --path crates/orbixdev
```

## Using orbixdev
Using `orbixdev` is simple. All you need to do is run the following command.
```shell
orbixdev --toml-file [path/to/TOML/file]
```

provide `orbixdev` with a TOML file containing the following keys:
```toml
username = "your_username"
password = "your_password"
dbname = "your_db_name"
```

Simply run the command and let `orbixdev` handle the rest.
