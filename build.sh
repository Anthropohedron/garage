#!/bin/sh

usage () {
	echo "Usage: $0 [-r] [-w]" >&2
	exit 1
}

WATCH=false
RELEASE=false
while test $# -gt 0
do
	case "x$1" in
		x-r) shift; RELEASE=true ;;
		x-w) shift; WATCH=true ;;
		*) usage ;;
	esac
done

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
if $RELEASE
then
	set -- "$@" --release
fi

exec cargo "$@"
