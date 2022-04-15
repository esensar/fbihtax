#!/bin/sh

# This script should be run via curl:
#   sudo sh -c "$(curl -fsSL https://raw.githubusercontent.com/esensar/fbihtax/main/scripts/install.sh)"
# or via wget:
#   sudo sh -c "$(wget -qO- https://raw.githubusercontent.com/esensar/fbihtax/main/scripts/install.sh)"
# or via fetch:
#   sudo sh -c "$(fetch -o - https://raw.githubusercontent.com/esensar/fbihtax/main/scripts/install.sh)"
#
# arguments can be passed as environment variables
# FBIHTAX_VERSION
# PDFTK_VERSION
# SKIP_PDFTK

set -e

if [ "$(id -u)" -ne 0 ]; then
    echo 'This script must be run with root privileges' >&2
    exit 1
fi

PDFTK_VERSION="${PDFTK_VERSION:-3.3.2}"
FBIHTAX_VERSION="${FBIHTAX_VERSION:-latest}"
SKIP_PDFTK="${SKIP_PDFTK:-false}"

if [ "$SKIP_PDFTK" = "false" ]
then
	wget -O /usr/bin/pdftk "https://gitlab.com/api/v4/projects/5024297/packages/generic/pdftk-java/v${PDFTK_VERSION}/pdftk" && chmod a+x /usr/bin/pdftk
fi

if [ "$FBIHTAX_VERSION" = "latest" ]
then
	wget -O fbihtax-linux.zip "https://github.com/esensar/fbihtax/releases/latest/download/fbihtax-linux.zip" && unzip fbihtax-linux.zip && mv fbihtax /usr/bin/fbihtax && chmod a+x /usr/bin/fbihtax && rm fbihtax-linux.zip
else
	wget -O fbihtax-linux.zip "https://github.com/esensar/fbihtax/releases/download/${FBIHTAX_VERSION}/fbihtax-linux.zip" && unzip fbihtax-linux.zip && mv fbihtax /usr/bin/fbihtax && chmod a+x /usr/bin/fbihtax && rm fbihtax-linux.zip
fi
