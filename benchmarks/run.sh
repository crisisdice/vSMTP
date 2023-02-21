#!/bin/bash

set -e

function run_postfix() {
    bench="$1"
    sessions="$2"
    length="$3"
    messages="$4"

    postfix -c "/etc/postfix-$bench" start
    mesure_session "postfix-$bench" $sessions $length $messages
    postfix -c "/etc/postfix-$bench" stop
}

function run_vsmtp() {
    bench="$1"
    sessions="$2"
    length="$3"
    messages="$4"

    vsmtp -c "/etc/vsmtp/benchmarks/$bench/vsmtp.vsl" --no-daemon &
    sleep 2

    run_benchmarks "vsmtp-$bench" $sessions $length $messages

    kill %%
}

function mesure_session() {
    server="$1"
    sessions="$2"
    length="$3"
    messages="$4"

    result=$(time smtp-source -s $sessions -l $length -m $messages -f john.doe@example.com -N -t jane.doe@example.com 127.0.0.1:25)

    echo "[$server] $result"
}

smtp-sink -u postfix -c 127.0.0.1:10025 100000 &

run_postfix $1
run_vsmtp $1

kill %%
