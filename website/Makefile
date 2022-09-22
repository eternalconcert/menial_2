#!/usr/bin/make

clean:
	rm -rf website/build/*

build: clean
	@mkdir -p website/build/styles
	anvil -i src/ -s src/less/ -o build/ -t "menial 2"

deploy: build
	scp -r build/* christian@softcreate.de://mnt/volumes/menial/

.PHONY: clean build deploy
