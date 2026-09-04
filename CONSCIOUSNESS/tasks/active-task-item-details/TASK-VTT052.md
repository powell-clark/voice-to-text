# TASK-VTT052: Design CT2 persistent daemon protocol

## Acceptance Criteria
1. Protocol spec is documented: stdin/stdout line-delimited JSON with request/response shapes defined
2. Spec covers: load_model, transcribe (with audio path), status, shutdown commands
3. Health-check heartbeat interval and failure detection strategy are specified
4. ADR filed capturing the decision to use line-delimited JSON over gRPC or Unix socket
