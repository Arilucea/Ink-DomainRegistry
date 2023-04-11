export salt = $(shell date  +%s)

build:
	cd domain_registry && cargo contract build

deploy-testnet: build
	cd domain_registry && cargo contract instantiate --constructor new --suri //Alice --salt $(salt) --skip-confirm

execute-test:
	cp domain_registry/target/ink/metadata.json test/contract-files/
	cd test && yarn run test

deploy-and-test: deploy-testnet execute-test