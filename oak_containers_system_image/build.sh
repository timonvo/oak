#!/bin/bash

set -o xtrace
set -o errexit
set -o nounset
set -o pipefail

readonly SCRIPTS_DIR="$(dirname "$0")"

cd "$SCRIPTS_DIR"

readonly IMAGE_BINARIES_DIRECTORY=./target/image_binaries

mkdir --parent "$IMAGE_BINARIES_DIRECTORY"

# build the orchestrator binary
cargo build --package=oak_containers_orchestrator --profile=release-lto --target=x86_64-unknown-linux-musl -Z unstable-options --out-dir=./target
cargo build --package=oak_containers_syslogd --profile=release-lto -Z unstable-options --out-dir=./target

sha256sum ./target/oak_containers_orchestrator
sha256sum ./target/oak_containers_syslogd

# We need to patch the binary to set the interpreter to the correct location, but we can't do it in-place, as that would
# confuse cargo. Therefore we copy the binary to a new location and patch that.
cp ./target/oak_containers_syslogd "$IMAGE_BINARIES_DIRECTORY"
cp ./target/oak_containers_orchestrator "$IMAGE_BINARIES_DIRECTORY"

# When built under nix the interpreter points to some Nix-specific location that doesn't exist on a regular Linux host, therefore
# we need to manually patch the binary to set it back to the normal regular location.
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "./$IMAGE_BINARIES_DIRECTORY/"oak_containers_syslogd
# The RUNPATH ELF section of the produced binary tends to contain references to paths on the host
# machine, outside of the nix environment. The "--shrink-rpath" removes those.
patchelf --shrink-rpath --allowed-rpath-prefixes "/nix/" "./$IMAGE_BINARIES_DIRECTORY/"oak_containers_syslogd

sha256sum "./$IMAGE_BINARIES_DIRECTORY/"oak_containers_orchestrator
sha256sum "./$IMAGE_BINARIES_DIRECTORY/"oak_containers_syslogd

bazel build oak_containers_system_image

cp ../bazel-bin/oak_containers_system_image/oak_containers_system_image.tar ./target/image.tar
sha256sum target/image.tar

xz --force target/image.tar
sha256sum target/image.tar.xz
