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

CREATE VIEW "overview" AS select
	measurements.measurement_id,
	measurements.timestamp,
	
	printf('%s %s (%s)', 
		json_extract(measurements.server_json, '$.name'),
		json_extract(measurements.server_json, '$.location'),
		json_extract(measurements.server_json, '$.host')
	) as server_info,
	
	round(avg(json_extract(records.details_json, '$.download.bandwidth')) / 125000.0, 2) as download_bandwidth,
	round(avg(json_extract(records.details_json, '$.upload.bandwidth')) / 125000.0, 2) as upload_bandwidth,
	round(avg(json_extract(records.details_json, '$.ping.latency')), 2) as latency
from
	measurements
inner join
	records on measurements.measurement_id = records.measurement_id
group by
	records.measurement_id

COMMIT;
