#! /usr/bin/bash

cargo b
./res/maelstrom/maelstrom test -w echo --bin ./target/debug/raita --node-count 1 --time-limit 10
