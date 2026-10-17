.PHONY: build serve clean

build:
	trunk build --release

serve:
	trunk serve

clean:
	rm -rf dist
	trunk clean
