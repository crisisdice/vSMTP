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

function setup_dkim_for_postfix() {
    domain="$1"

    apt install opendkim opendkim-tools

    # Generate dkim keys.
    mkdir -p /etc/postfix-dkim-dmarc/dkim
    opendkim-genkey -D /etc/postfix-dkim-dmarc/dkim/ -d "$domain" -s mail
    chgrp opendkim /etc/postfix-dkim-dmarc/dkim/*
    chmod g+r /etc/postfix-dkim-dmarc/dkim/*

    # Create postfix key table.
    echo "mail._domainkey.$domain $domain:mail:/etc/postfix-dkim-dmarc/dkim/mail.private" >/etc/postfix-dkim-dmarc/dkim/keytable

    # Create a signing table.
    echo "*@$domain mail._domainkey.$domain" >/etc/postfix-dkim-dmarc/dkim/signingtable

    # Add trusted hosts.
    echo "127.0.0.1
        10.1.0.0/16
        1.2.3.4/24" >/etc/postfix-dkim-dmarc/dkim/trustedhosts

    # Setup opendkim.
    echo "KeyTable file:/etc/postfix-dkim-dmarc/dkim/keytable
        SigningTable refile:/etc/postfix-dkim-dmarc/dkim/signingtable
        InternalHosts refile:/etc/postfix-dkim-dmarc/dkim/trustedhosts

        Canonicalization        relaxed/simple
        Socket                  inet:12301@localhost" >/etc/opendkim.conf

    # Interfacing with postfix.
    postmulti -i postfix-dkim-dmarc -x postconf -e "myhostname = $(cat /etc/mailname)"
    postmulti -i postfix-dkim-dmarc -x postconf -e "milter_default_action = accept"
    postmulti -i postfix-dkim-dmarc -x postconf -e "milter_protocol = 6"
    postmulti -i postfix-dkim-dmarc -x postconf -e "smtpd_milters = inet:localhost:12301"
    postmulti -i postfix-dkim-dmarc -x postconf -e "non_smtpd_milters = inet:localhost:12301"

    systemctl restart opendkim
    systemctl enable opendkim

    echo "[postfix-dkim-dmarc] Public key generated (to paste to a DNS TXT entry in your registrar):"
    echo -e "

v=DKIM1; k=rsa; $(tr -d "
" </etc/postfix-dkim-dmarc/dkim/mail.txt | sed "s/k=rsa.* \"p=/k=rsa; p=/;s/\"\s*\"//;s/\"\s*).*//" | grep -o "p=.*")

"
}

setup_dkim_for_postfix $1
