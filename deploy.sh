#! /bin/bash
cargo lambda build --release --arm64
cargo lambda deploy --profile brscans