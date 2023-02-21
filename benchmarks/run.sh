#!/bin/bash

function run_postfix() {
    bench="$1"

    postfix -c "postfix-$bench" start
    send_emails_and_mesure "postfix-$bench"
    postfix -c "postfix-$bench" stop
}

function run_vsmtp() {
    bench="$1"

    vsmtp -c /etc/vsmtp/benchmarks/"$bench" --no-deamon &

    send_emails_and_mesure "vsmtp-$bench"

    kill %%
}

function send_emails_and_mesure() {
    server="$1"

    smtp-sink -c 127.0.0.1:10025 100000
    result=$(time smtp-source -s 4 -l 10000 -m 100000 -f john.doe@example.com -N -t jane.doe@example.com 127.0.0.1:25)

    echo "[$server] $result"
}

run_postfix $1
run_vsmtp $1
