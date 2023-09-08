create table tokens( index INTEGER PRIMARY KEY, id TEXT,count INTEGER,bracket integer,level text);
create table info(hash TEXT PRIMARY KEY, wbgl INTEGER);
CREATE TABLE info_lotto(last_payment TEXT PRIMARY KEY, wbgl INTEGER,wining_block INTEGER, round INTEGER);