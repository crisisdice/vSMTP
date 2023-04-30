##
FROM vsmpt-base AS base
FROM debian AS runtime

# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# apt install pkg-config build-essential sasl2-bin

RUN apt-get update && apt-get install -y
# RUN apt-get install vim python3 telnet net-tools mail -y
RUN apt-get install mailutils -y

RUN addgroup vsmtp && \
    adduser --shell /sbin/nologin --disabled-password \
    --no-create-home --ingroup vsmtp vsmtp

RUN mkdir /var/log/vsmtp/ && chown vsmtp:vsmtp /var/log/vsmtp/ && chmod 755 /var/log/vsmtp/
RUN mkdir /var/spool/vsmtp/ && chown vsmtp:vsmtp /var/spool/vsmtp/ && chmod 755 /var/spool/vsmtp/
RUN mkdir /etc/vsmtp/ && chown vsmtp:vsmtp /etc/vsmtp/ && chmod 755 /etc/vsmtp/
# RUN mkdir /etc/vsmtp/plugins && chown vsmtp:vsmtp /etc/vsmtp/plugins && chmod 755 /etc/vsmtp/plugins

COPY --from=base /root/.cargo/bin/vsmtp /usr/sbin/vsmtp

#USER vsmtp

#RUN /usr/sbin/vsmtp --version
#CMD ["/usr/sbin/vsmtp", "-c", "/etc/vsmtp/vsmtp.vsl", "--no-daemon", "--stdout"]
