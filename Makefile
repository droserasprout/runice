lint:
	rustfmt src/*
	cargo clippy

install:
	cargo build
	sudo rm -rf /etc/runice/
	sudo mkdir /etc/runice
	sudo cp -r config/* /etc/runice/
	sudo ./target/debug/runice import-ananicy