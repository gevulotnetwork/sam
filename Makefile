test: build
	./target/debug/sam -c examples/self-test/config.yaml

build: target/debug/sam

target/debug/sam:
	cargo build

release: clean
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --target x86_64-unknown-linux-gnu --release

install: release
	sudo cp target/x86_64-unknown-linux-gnu/release/sam /usr/local/bin

clean:
	cargo clean