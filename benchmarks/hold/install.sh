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

set -e

postmulti -i postfix-hold -x postconf -e "smtpd_banner = \$myhostname ESMTP \$mail_name"
postmulti -i postfix-hold -x postconf -e "smtpd_client_restrictions = permit_mynetworks"
postmulti -i postfix-hold -x postconf -e "smtpd_recipient_restrictions = static:hold"
postmulti -i postfix-hold -x postconf -e "message_size_limit = 200000000"
postmulti -i postfix-hold -x postconf -e "myorigin = \$mydomain"
postmulti -i postfix-hold -x postconf -e "mynetworks = 127.0.0.0/24"
