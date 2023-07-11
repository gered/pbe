#!/bin/bash
cd "$(dirname "$0")"

package_version=$(cargo pkgid | cut -d# -f2 | cut -d: -f2)
current_target=$(rustc -vV | sed -n 's|host: ||p')

package_filename="pbe-v${package_version}-${current_target}.tar.gz"

rm -f ./artifacts/bin/pbe
rm -f ./artifacts/bin/syntax_to_css

cargo clean
cargo install --path . --root ./artifacts/
cargo install --path ./syntax_to_css/ --root ./artifacts/

tar -cvf $package_filename \
    example-site \
    -C ./artifacts/bin \
    pbe \
    syntax_to_css

echo "Packaged release as ${package_filename}"
