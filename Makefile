.PHONY: default build build-dev build-prod build-release build-debug dev dev-prod release debug check check-dev check-prod lint-design-system clean

# Default profile is 'dev'
default: dev
build: build-dev

dev: build-dev
build-dev:
	trunk build

prod: build-prod
build-prod:
	trunk build --release

release: build-release
build-release:
	trunk build --release

# Debug build is explicit and not run unless asked
debug: build-debug
build-debug:
	trunk build

# Dev server (http://127.0.0.1:8080). Matches honba-fe's `make dev` naming.
serve:
	trunk serve

serve-prod:
	trunk serve --release

check: check-dev lint-design-system

check-dev:
	cargo check --profile dev --workspace

check-prod:
	cargo check --profile prod --workspace

# Prohibit raw <button>/<input> elements in the app crate: all interactive
# primitives must come from honba-kao-ui-core (HfButton, HfInput, etc.).
lint-design-system:
	@! grep -rnE '<(button|input)[ >]' crates/honba-kao-app/src/features/ crates/honba-kao-app/src/router/ crates/honba-kao-app/src/shared/components/ \
		&& echo "lint-design-system: no raw <button>/<input> found" \
		|| (echo "lint-design-system: raw <button>/<input> found above - use honba_kao_ui_core primitives instead" && exit 1)

clean:
	rm -rf dist target
	trunk clean
