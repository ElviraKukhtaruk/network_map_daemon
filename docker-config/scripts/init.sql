-- Read-only user for DICTIONARY
CREATE USER client_readonly IDENTIFIED WITH no_password SETTINGS PROFILE 'readonly';
GRANT SHOW TABLES, SELECT ON stats.* TO client_readonly;

CREATE TABLE IF NOT EXISTS stats.stat(
    `timestamp` DateTime() CODEC(DoubleDelta),
    `uuid` LowCardinality(UUID),
    `interface` LowCardinality(String),
    `rx` UInt64,
    `tx` UInt64,
    `rx_p` UInt64,
    `tx_p` UInt64 -- ZSTD, Delta/DoubleDelta
)
ENGINE = MergeTree
PARTITION BY uuid
ORDER BY (timestamp, uuid, interface);

CREATE TABLE IF NOT EXISTS stats.addr(
    `uuid` LowCardinality(UUID),
    `hostname` LowCardinality(String),
    `interface` LowCardinality(String),
    `prefix_lenv6` Array(UInt8),
    `IPv6` Array(IPv6) CODEC(ZSTD)
)
ENGINE = MergeTree
ORDER BY (uuid, interface, IPv6);

CREATE DICTIONARY stats.hostname(
    `uuid` UUID,
    `hostname` String
)
PRIMARY KEY uuid
SOURCE(CLICKHOUSE(USER client_readonly TABLE addr))
LIFETIME(100)
LAYOUT(COMPLEX_KEY_HASHED_ARRAY);