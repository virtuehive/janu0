
#
# Copyright (c) 2017, 2020 ADLINK Technology Inc.
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ADLINK janu team, <janu@adlink-labs.tech>
#

# Script generating the "janu" top-level Debian package

if [ -z "$1" -o -z "$2" ]; then
    echo "Usage: $0 VERSION ARCH"
    echo "  example: $0 0.5.0-beta.9 amd64"
    exit 1
fi

VERSION=`echo $1 | sed s/-/~/g`
ARCH=$2

PACKAGE_NAME="janu_${VERSION}_${ARCH}"
CONTROL_FILE="${PACKAGE_NAME}/DEBIAN/control"

echo "Generate janu top-level package: ${PACKAGE_NAME}.deb ..."
# create control file for janu deb package
mkdir -p ${PACKAGE_NAME}/DEBIAN
echo "Package: janu " > ${CONTROL_FILE}
echo "Version: ${VERSION} " >> ${CONTROL_FILE}
echo "Architecture: ${ARCH}" >> ${CONTROL_FILE}
echo "Vcs-Browser: https://github.com/eclipse-janu/janu" >> ${CONTROL_FILE}
echo "Vcs-Git: https://github.com/eclipse-janu/janu" >> ${CONTROL_FILE}
echo "Homepage: http://janu.io" >> ${CONTROL_FILE}
echo "Section: net " >> ${CONTROL_FILE}
echo "Priority: optional" >> ${CONTROL_FILE}
echo "Essential: no" >> ${CONTROL_FILE}
echo "Installed-Size: 1024 " >> ${CONTROL_FILE}
echo "Depends: janud, janu-plugin-rest, janu-plugin-storages " >> ${CONTROL_FILE}
echo "Maintainer: janu-dev@eclipse.org " >> ${CONTROL_FILE}
echo "Description: The janu top-level package" >> ${CONTROL_FILE}
echo "" >> ${CONTROL_FILE}

dpkg-deb --build ${PACKAGE_NAME}
