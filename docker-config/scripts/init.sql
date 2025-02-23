-- Read-only user
CREATE USER client_readonly IDENTIFIED WITH no_password SETTINGS PROFILE 'readonly';
GRANT SHOW TABLES, SELECT ON stats.* TO client_readonly;

CREATE TABLE IF NOT EXISTS stats.stat(
    `server_id` LowCardinality(FixedString(32)),
    `interface` LowCardinality(String),
    `timestamp` DateTime() CODEC(DoubleDelta),
    `rx` UInt64,
    `tx` UInt64,
    `rx_p` UInt64,
    `tx_p` UInt64,
    `rx_d` UInt64,
    `tx_d` UInt64
)
ENGINE = MergeTree
PARTITION BY (server_id, interface, toYYYYMM(timestamp))
PRIMARY KEY (server_id, interface)
ORDER BY (server_id, interface, timestamp);

CREATE TABLE IF NOT EXISTS stats.server(
    `server_id` LowCardinality(FixedString(32)),
    `hostname` LowCardinality(String),
    `icao` LowCardinality(Nullable(String)),
    `lat` Float64 NULL,
    `lng` Float64 NULL,
    `city` LowCardinality(Nullable(String)),
    `country` LowCardinality(Nullable(String)),
)
ENGINE = MergeTree
PRIMARY KEY (server_id, hostname)
ORDER BY (server_id, hostname);

CREATE TABLE IF NOT EXISTS stats.addr(
    `server_id` LowCardinality(FixedString(32)),
    `interface` LowCardinality(String),
    `IPv6` Array(Tuple(Nullable(IPv6), Nullable(UInt8))) CODEC(ZSTD),
    `IPv6_peer` Array(Tuple(Nullable(IPv6), Nullable(UInt8))) CODEC(ZSTD)
)
ENGINE = MergeTree
PRIMARY KEY (server_id, interface)
ORDER BY (server_id, interface);

CREATE DICTIONARY stats.interface(
    `server_id` String,
    `interface` String
)
PRIMARY KEY (server_id, interface)
SOURCE(CLICKHOUSE(USER client_readonly TABLE addr))
LIFETIME(10)
LAYOUT(COMPLEX_KEY_HASHED_ARRAY);
