worker:
	cargo run -- worker --target storage

master:
	cargo run -- master --source source

node:
	cargo run -- node add --ips=0.0.0.0:50051

node-list:
	cargo run -- node list

help:
	cargo run -- -h
