BASENAME = $(shell basename $(shell pwd))

compile:
	docker run --rm -v "$(shell pwd)":/code --mount type=volume,source="$(BASENAME)_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.12.10

clippy:
	cargo clippy

test:	
	cargo test -- --nocapture

# Requires juno test_node.sh to be running locally
e2e:
	sh test.sh