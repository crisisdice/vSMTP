#!/bin/bash

function run_postfix() {
    sudo systemctl start postfix
    time smtp-source -s 4 -l 10000 -m 100000 -f john.doe@example.com -N -t jane.doe@example.com 127.0.0.1:25
    sudo systemctl stop postfix
    rm /var/spool/postfix/hold/*
}

function run_vsmtp() {
    sudo systemctl start vsmtp
    time smtp-source -s 4 -l 10000 -m 100000 -f john.doe@example.com -N -t jane.doe@example.com 127.0.0.1:25
    sudo systemctl stop vsmtp
    rm -rf /var/spool/vsmtp/
}

smtp-sink -c 127.0.0.1:10025 100000

run_postfix
run_vsmtp
