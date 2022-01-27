#!/usr/bin/env bash

INK_EXAMPLES_PATH=./contracts RUST_LOG=debug cargo test --features polkadot-js-ui --features headless
