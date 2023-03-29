export salt = $(shell date  +%s)

build:
	cd domain_registry && cargo contract build

deploy-testnet: build
	cd domain_registry && cargo contract instantiate --constructor new --suri //Alice --salt $(salt) --skip-confirm

