#!/bin/bash

function build_bpf() {
    cargo build-bpf --manifest-path=program/Cargo.toml --bpf-out-dir=dist/program
}

case $1 in
    "build-bpf")
	build_bpf
	;;
    "deploy")
	build_bpf
	solana program deploy dist/program/associated_multisig_account.so
	solana program deploy dist/program/multisig.so
	;;
    "client")
	(cd client/; cargo run)
	;;
    "clean")
	(cd program/; cargo clean)
	(cd client/; cargo clean)
	rm -rf dist/
	;;
    *)
	echo "usage: $0 build-bpf"
	;;
esac
