.PHONY: default build build-dev build-prod build-release build-debug dev dev-prod release debug check check-dev check-prod lint-design-system lint-a11y clean

# Default profile is 'prod' (no debug build unless asked for specifically)
default: prod
build: build-prod

prod: build-prod
build-prod:
	trunk build --config crates/honba-kao-app/Trunk.toml --release

release: build-release
build-release:
	trunk build --config crates/honba-kao-app/Trunk.toml --release

dev: build-dev
build-dev:
	trunk build --config crates/honba-kao-app/Trunk.toml

# Debug build is explicit and not run unless asked
debug: build-debug
build-debug:
	trunk build --config crates/honba-kao-app/Trunk.toml

# Dev server (http://127.0.0.1:8080). Defaults to release mode.
serve: serve-prod

serve-prod:
	trunk serve --config crates/honba-kao-app/Trunk.toml --release

serve-dev:
	trunk serve --config crates/honba-kao-app/Trunk.toml

check: check-prod lint-design-system lint-a11y

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

# Accessibility: WCAG AA contrast tokens, roving keyboard navigation, ARIA
# roles, focus-visible policy. Static check (no browser engine offline).
lint-a11y:
	./scripts/check-a11y.sh

clean:
	rm -rf dist target crates/honba-kao-app/dist
	trunk clean --config crates/honba-kao-app/Trunk.toml 2>/dev/null || true
