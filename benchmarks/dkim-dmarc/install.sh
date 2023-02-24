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

function setup_postfix() {
    postmulti -i postfix-dkim-dmarc -x postconf -e "smtpd_banner = \$myhostname ESMTP \$mail_name"
    postmulti -i postfix-dkim-dmarc -x postconf -e "smtpd_client_restrictions = permit_mynetworks"
    postmulti -i postfix-dkim-dmarc -x postconf -e "message_size_limit = 200000000"
    postmulti -i postfix-dkim-dmarc -x postconf -e "myorigin = \$mydomain"
    postmulti -i postfix-dkim-dmarc -x postconf -e "mynetworks = 127.0.0.0/24"
    postmulti -i postfix-dkim-dmarc -x postconf -e "relay_domains = [127.0.0.1]:10025"
    postmulti -i postfix-dkim-dmarc -x postconf -e "relayhost = [127.0.0.1]:10025"
    postmulti -i postfix-dkim-dmarc -x postconf -e "inet_interfaces = loopback-only"
    postmulti -i postfix-dkim-dmarc -x postconf -e "local_recipient_maps ="
}

function setup_dkim() {
    domain="$1"

    echo "[dkim-dmarc] Setting up DKIM."

    echo "[dkim-dmarc] install opendkim"
    apt install -y opendkim opendkim-tools

    echo "[dkim-dmarc] Generate dkim keys"
    mkdir -p /etc/postfix-dkim-dmarc/dkim
    opendkim-genkey -D /etc/postfix-dkim-dmarc/dkim/ -d "$domain" -s mail
    chgrp opendkim /etc/postfix-dkim-dmarc/dkim/*
    chmod g+r /etc/postfix-dkim-dmarc/dkim/*

    echo "[dkim-dmarc] Create postfix key table."
    echo "mail._domainkey.$domain $domain:mail:/etc/postfix-dkim-dmarc/dkim/mail.private" >/etc/postfix-dkim-dmarc/dkim/keytable

    echo "[dkim-dmarc] Create a signing table."
    echo "*@$domain mail._domainkey.$domain" >/etc/postfix-dkim-dmarc/dkim/signingtable

    echo "[dkim-dmarc] Add trusted hosts."
    echo "127.0.0.1
        10.1.0.0/16
        1.2.3.4/24" >/etc/postfix-dkim-dmarc/dkim/trustedhosts

    echo "[dkim-dmarc] Setup opendkim."
    echo "KeyTable file:/etc/postfix-dkim-dmarc/dkim/keytable
        SigningTable refile:/etc/postfix-dkim-dmarc/dkim/signingtable
        InternalHosts refile:/etc/postfix-dkim-dmarc/dkim/trustedhosts

        Canonicalization        relaxed/simple
        Socket                  inet:12301@localhost" >>/etc/opendkim.conf

    echo "[dkim-dmarc] Interfacing with postfix."
    postmulti -i postfix-dkim-dmarc -x postconf -e "myhostname = $domain"
    postmulti -i postfix-dkim-dmarc -x postconf -e "milter_default_action = accept"
    postmulti -i postfix-dkim-dmarc -x postconf -e "milter_protocol = 6"
    postmulti -i postfix-dkim-dmarc -x postconf -e "smtpd_milters = inet:127.0.0.1:12301"
    postmulti -i postfix-dkim-dmarc -x postconf -e "non_smtpd_milters = inet:127.0.0.1:12301"

    echo "[dkim-dmarc] Launch opendkim."
    opendkim -f &

    echo "[postfix-dkim-dmarc] Public key generated (to paste to a DNS TXT entry in your registrar):"
    echo "Host field: mail._domainkey.$domain"
    echo -e "TXT value: v=DKIM1; k=rsa; $(tr -d "
" </etc/postfix-dkim-dmarc/dkim/mail.txt | sed "s/k=rsa.* \"p=/k=rsa; p=/;s/\"\s*\"//;s/\"\s*).*//" | grep -o "p=.*")
"
}

function setup_dmarc() {
    domain="$1"

    echo "[dkim-dmarc] Setting up DMARC."

    useradd -m -G mail dmarc

    echo "[dkim-dmarc] dmarc record generated (to paste to a DNS TXT entry in your registrar):"
    echo "Host field: _dmarc.$domain"
    echo "TXT value: v=DMARC1; p=reject; rua=mailto:dmarc@$domain; fo=1"
}

function setup_spf() {
    echo "[dkim-dmarc] Setting up SPF."
    echo "[dkim-dmarc] spf record generated (to paste to a DNS TXT entry in your registrar):"
    echo "Host field: $domain"
    echo "TXT value: v=spf1 mx a:mail.$domain -all"
}

read -p "Domain of the server (example.com): " domain

setup_postfix
setup_dkim "$domain"
setup_dmarc "$domain"
setup_spf "$domain"
