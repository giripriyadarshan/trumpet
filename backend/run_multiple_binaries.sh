#!/usr/bin/env bash

for bin in "$@"
do
    nohup cargo run --bin "$bin" > "$bin".out &
done
