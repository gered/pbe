#!/bin/bash
cd "$(dirname "$0")"

cargo install --path . --root ./artifacts/
cargo install --path ./syntax_to_css/ --root ./artifacts/

cd ./artifacts/bin
tar -cvf ../../release.tar.gz ./pbe ./syntax_to_css

