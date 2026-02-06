.PHONY:

structure:
	@mkdir -p apps/api-server/src
	@mkdir -p apps/worker/src
	@mkdir -p crates/core/src/domain/entities
	@mkdir -p crates/core/src/domain/ports
	@mkdir -p crates/core/src/services
	@mkdir -p crates/adapters/src/persistence/postgres
	@mkdir -p libs/configuration/src
	@mkdir -p libs/telemetry/src
	@mkdir -p libs/common/src