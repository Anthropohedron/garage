#!/bin/sh

WATCH=false
if test $# -gt 0
then
	if test $# -eq 1 -a x"$1" = "x-w"
	then
		WATCH=true
	else
		echo "Usage: $0 [-w]" >&2
		exit 1
	fi
fi

CONFIG1='target.aarch64-unknown-linux-gnu.linker = "/usr/bin/aarch64-linux-gnu-gcc"'
set -- build \
	--target=aarch64-unknown-linux-gnu \
	--config

if $WATCH
then
	set -- watch "$@" "'$CONFIG1'"
else
	set -- "$@" "$CONFIG1"
fi
exec cargo "$@"
