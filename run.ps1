$env:RUST_LOG = 'debug'
$env:FILE_STORAGE_ADMIN_ACCESS_KEY = 'username'
$env:FILE_STORAGE_ADMIN_SECRET_KEY = 'secret'
# cargo run --release
cargo run
