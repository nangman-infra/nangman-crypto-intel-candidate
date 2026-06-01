include "diagnose-candidate-source-gaps-records";

def symbolized($records; $structured_records):
  [
    $records[] as $record
    | (($record | record_symbols) as $direct_symbols
      | if ($direct_symbols | length) > 0 then $direct_symbols
        else [
          $structured_records[]
          | select(packet_id == ($record | packet_id))
          | record_symbols[]
        ] | unique | sort
        end) as $symbols
    | $record + {
        __packet_id:($record | packet_id),
        __supersedes_packet_id:($record | supersedes_packet_id),
        __symbols:$symbols
      }
  ];

def current_records($records; $superseded_packet_ids):
  [
    $records[] as $record
    | select(
        ($record.__packet_id | length) == 0
        or (($superseded_packet_ids | index($record.__packet_id)) == null)
      )
    | $record
  ];

def superseded_records($records; $superseded_packet_ids):
  [
    $records[] as $record
    | select(
        ($record.__packet_id | length) > 0
        and (($superseded_packet_ids | index($record.__packet_id)) != null)
      )
    | $record
  ];

def records_for_symbol($records; $symbol):
  [
    $records[]
    | select((.__symbols // record_symbols) | index($symbol))
  ];
