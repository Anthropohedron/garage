#!/bin/sh

exec cargo build \
	--target=aarch64-unknown-linux-gnu \
	--config 'target.aarch64-unknown-linux-gnu.linker = "/usr/bin/aarch64-linux-gnu-gcc"'
