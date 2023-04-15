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

echo "=>  Run tests"

touch "greylist.db"

cat ./docker/sqlite-config/greylist-dump.sql | sqlite3 greylist.db

# Simply send an email twice, to verify the effects of the greylist.
reject=$(curl -vv -k --url 'smtp://localhost:10025' \
    --mail-from 'john.doe@example.com' --mail-rcpt 'jenny.doe@example.com' \
    --upload-file ./docker/tests/simple/test.eml 2>&1 | tail -n 1)

if [[ $(echo "$reject" | grep -Fi "451") ]]; then
    echo "First send command rejected with 451 code"
else
    echo "ERROR: first command did not get rejected by greylist."
    echo "$reject"
    exit 1
fi

accept=$(curl -vv -k --url 'smtp://localhost:10025' \
    --mail-from 'john.doe@example.com' --mail-rcpt 'jenny.doe@example.com' \
    --upload-file ./docker/tests/simple/test.eml 2>&1 | tail -n 3)

if [[ $(echo "$accept" | grep -Fi "250 Ok") ]]; then
    echo "Second send command accepted with 250 code"
    echo "$accept"
else
    echo "ERROR: second command did not get accepted by greylist."
    echo "$accept"
    exit 1
fi

rm "greylist.db"
