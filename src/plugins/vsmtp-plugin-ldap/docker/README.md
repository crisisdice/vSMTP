# Docker setup with ldap client plugin.

The following docker files installs an instance of vSMTP (using an alpine image) and an openldap database. (via the official images)

It is used to test the plugin in our CI environments, but you can launch your own instance via the following command: 

```
$ docker compose build
$ docker compose up
```

vsmtp is accessible via port `10025` on the host.
