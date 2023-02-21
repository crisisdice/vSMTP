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

# Download and intall both vsmtp and postfix.
sudo apt install postfix

curl -s https://api.github.com/repos/viridit/vsmtp/releases/latest |
    grep "browser_download_url.*ubuntu22.04_amd64.deb" |
    cut -d : -f 2,3 |
    tr -d \" |
    wget -qi -

sudo apt install ./vsmtp*

vsmtp --help

# Using postmulti to handle multiple postfix configuration.
postmulti -e init

# Copy vsmtp and postfix configurations for each benchmarks.
for bench in "hold" "dkim-dmarc"; do
    pb="postfix-$bench"

    postmulti -I "$pb" -G mta -e create
    cp -f "$bench"/postfix/* /etc/"$pb"/

    postmulti -i "$pb" -x postconf -e \
        "master_service_disable =" "authorized_submit_users = root"
    postmulti -i "$pb" -e enable
    # postmulti -i "$pb" -p start

    # vsmtp coonfigurations are simply stored in `etc`.
    cp -f "$bench"/vsmtp/* /etc/vsmtp/benchmarks/"$bench"
done
