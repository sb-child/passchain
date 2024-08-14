#!/bin/sh

cargo b -r || exit 1

cp target/release/passchain passchain || exit 1

upx -9 passchain
