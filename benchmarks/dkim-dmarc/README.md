# "DKIM-DMARC" benchmarks

This benchmark mesures time spent on email processing between vSMTP and Postfix when receiving multiple messages with dkim mechanisms setup for both programs.

## Install

Download the latest debian package from the vsmtp repository, then install it via `apt`.

```sh
apt install -y ./vsmtp.deb
```

Install Postfix.

```sh
apt install -y mailutils postfix
```

When asked for a "mail name", give your full domain name from which you would like mail to come and go, e.g. `example.org`.

Then, copy vsmtp and postfix configurations to their respective files.

> Do not forget to backup any existing configuration for both programs before copying the files.

```
cp postfix/main.cf /etc/postfix
cp -f vsmtp/ /etc/vsmtp
```

You can use `systemctl` to run postfix & vsmtp as services.

```sh
sudo systemctl start postfix.service
## or
sudo systemctl start vsmtp.service
```

## Run benchmarks

`smtp-source` is used to simulate email traffic, and `smtp-sink` is used to act as a receiving server. Both of those programs
are packaged with postfix.

> Before running any of the commands below, make sure that your Postfix and vSMTP queues and log directory are empty. If not, make sur to make backups of those directories, located in `/var/spool`.

You can use the following command to simulate incoming clients.

```sh
time smtp-source -s <nbr-of-sessions>    \
                 -l <message-size>       \
                 -m <nbr-of-messages>    \
                 -f <sender-address>     \
                 -N                      \
                 -t <recipient-address>  \
                 127.0.0.1:25
```

For example:

```sh
time smtp-source -s 4 -l 1000000 -m 10000 -f john.doe@example.com -N -t jane.doe@example.com 127.0.0.1:25
```
