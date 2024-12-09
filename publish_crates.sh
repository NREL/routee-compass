#!/bin/bash

# exit on failure
set -e

cd rust/

cargo publish -p routee-compass-core --dry-run
cargo publish -p routee-compass-core

cargo publish -p routee-compass-powertrain --dry-run
cargo publish -p routee-compass-powertrain

cargo publish -p routee-compass --dry-run
cargo publish -p routee-compass

cargo publish -p routee-compass-macros --dry-run
cargo publish -p routee-compass-macros

cargo publish -p routee-compass-py --dry-run
cargo publish -p routee-compass-py