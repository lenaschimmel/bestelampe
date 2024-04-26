#!/bin/bash
echo "Prefix syntax: input voltage;number of LEDs in series;inductor in mH;full driver current in mA;switching frequency in kHz"
echo "Current benchmark prefix: $BENCHMARK_PREFIX"
cargo run | grep --line-buffered "line;" | tee /dev/tty | awk -v prefix="$BENCHMARK_PREFIX" '{ print prefix, $0 }' >> log.csv 