BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS "measurements" (
	"measurement_id"	INTEGER NOT NULL UNIQUE,
	"timestamp"	TEXT NOT NULL,
	"server_json" TEXT NOT NULL,
	PRIMARY KEY("measurement_id" AUTOINCREMENT)
) STRICT;

CREATE TABLE IF NOT EXISTS "records" (
	"record_id"	INTEGER NOT NULL UNIQUE,
	"measurement_id" INTEGER NOT NULL,
	"type" TEXT NOT NULL,
	"details_json" TEXT NOT NULL,
	PRIMARY KEY("record_id" AUTOINCREMENT),
	FOREIGN KEY("measurement_id") REFERENCES "measurements"("measurement_id")
) STRICT;

CREATE INDEX "measurement_records" ON "records" (
	"record_id",
	"measurement_id"
);

COMMIT;
