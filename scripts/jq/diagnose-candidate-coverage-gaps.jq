include "diagnose-candidate-coverage-gaps-lib";
include "diagnose-candidate-coverage-gaps-report";

require_supported_input as $status
| coverage_gap_diagnosis($status; $generated_at; $status_file)
