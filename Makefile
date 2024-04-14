actor-simple:
	RUST_LOG=debug cargo run --release -q --bin actor-simple

actor-simple-mutex:
	RUST_LOG=debug cargo run --release -q --bin actor-simple -- --mutex
