
.PHONY: .build deploy run

.build: 
	cargo build --target=armv7-unknown-linux-gnueabihf

deploy: .build
	scp -i ~/.ssh/pi2.pub ./target/armv7-unknown-linux-gnueabihf/debug/raspberry pi@pi2.local:/home/pi/Documents/Rust/

run: deploy
	ssh -i ~/.ssh/pi2.pub pi@pi2.local "~/Documents/Rust/raspberry"


