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
	solana program deploy dist/program/solana_multisig.so
	;;
    "client")
	(cd client/ || exit; shift && cargo run "$@")
	;;
    "clean")
	(cd program/ || exit; cargo clean)
	(cd client/ || exit; cargo clean)
	rm -rf dist/
	;;
    *)
	echo "usage: $0 build-bpf"
	;;
esac
