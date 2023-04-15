CREATE TABLE IF NOT EXISTS greylist_sender(
    address varchar(500) NOT null primary key,
    user varchar(500) NOT null,
    domain varchar(500) NOT null
)
