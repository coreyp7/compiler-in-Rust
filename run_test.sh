#!/bin/bash
cargo build
./compile.sh -i ./example.plank -o . --debug -D
#./compile.sh -i ./example.plank -o . -D