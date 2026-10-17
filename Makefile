.PHONY: build build-dev build-prod serve serve-dev serve-prod check check-dev check-prod clean

# Default profile is 'dev'
build: build-dev

build-dev:
	trunk build

build-prod:
	trunk build --release

serve: serve-dev

serve-dev:
	trunk serve

serve-prod:
	trunk serve --release

check: check-dev

check-dev:
	cargo check --profile dev --workspace

check-prod:
	cargo check --profile prod --workspace

clean:
	rm -rf dist
	trunk clean
