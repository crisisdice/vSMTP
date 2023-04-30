##
FROM vsmpt-base AS base
FROM debian AS runtime

RUN apt-get update && apt-get install -y
RUN apt-get install vim python3 telnet net-tools -y

# RUN apk upgrade --no-cache && apk add --no-cache libc6-compat

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
