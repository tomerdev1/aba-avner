.PHONY: smoke smoke-full release-check test test-backend test-ui test-visual build-ui package deploy-snap clean-generated coverage open-coverage

smoke:
	./scripts/test-all.sh

smoke-full:
	./scripts/smoke-full.sh

release-check:
	./scripts/release-check.sh

test:
	$(MAKE) smoke

test-backend:
	cd src-tauri && cargo test

test-ui:
	cd ui && npm test

test-visual:
	cd ui && npm run test:visual

build-ui:
	cd ui && npm run build

package:
	cd src-tauri && cargo tauri build

deploy-snap:
	sudo ./scripts/deploy-snap.sh $(CHANNEL)

clean-generated:
	node ./scripts/clean-generated.mjs

coverage:
	./scripts/coverage.sh

open-coverage:
	./scripts/open-coverage.sh
