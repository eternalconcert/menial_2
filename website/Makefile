#!/usr/bin/make

clean:
	rm -rf ../target/release/
	rm -f ../website/src/static/menial_2.bin
	rm -rf website/build/*

bin: clean
	cd ..
	cargo build -r
	mv ../target/release/main ../website/src/static/menial_2.bin
	python3 utils/update_values.py dev

src:
	tar -zcvf menial_2.tar.gz ../src/
	mv menial_2.tar.gz ../website/src/static/

build: bin
	@mkdir -p ../website/build/styles
	anvil -i src/ -s src/less/ -o build/ -t "menial 2" -v ../website/hashvalues.json

deploy: build
	scp -r build/* christian@softcreate.de://mnt/volumes/menial/

.PHONY: clean build deploy bin src
