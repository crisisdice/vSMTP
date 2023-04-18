#!/bin/bash

# vSMTP mail transfer agent
# Copyright (C) 2022 viridIT SAS
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or any later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see https://www.gnu.org/licenses/.

echo "=>  Building images"
docker compose build

echo "=>  Run containers"
docker compose up -d --remove-orphans --wait

echo "=>  Run tests"

echo "=>      reject first time connection"
# Simply send an email twice, to verify the effects of the greylist.
reject=$(curl -vv -k --url 'smtp://127.0.0.1:10025' \
    --mail-from 'john.doe@example.com' --mail-rcpt 'jenny.doe@example.com' \
    --upload-file ./test.eml 2>&1)

if [[ $(echo "$reject" | tail -n 1 | grep -Fi "451") ]]; then
    echo "First send command rejected with 451 code"
else
    echo "ERROR: first command did not get rejected by greylist."
    echo "$reject"
    exit 1
fi

echo "=>      accept second connection"
accept=$(curl -vv -k --url 'smtp://127.0.0.1:10025' \
    --mail-from 'john.doe@example.com' --mail-rcpt 'jenny.doe@example.com' \
    --upload-file ./test.eml 2>&1)

if [[ $(echo "$accept" | tail -n 3 | grep -Fi "250 Ok") ]]; then
    echo "Second send command accepted with 250 code"
    echo "$accept"
else
    echo "ERROR: second command did not get accepted by greylist."
    echo "$accept"
    exit 1
fi

echo "=>  Shuting down"
docker compose down
