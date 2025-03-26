This is an attempt to create a remote control for besteLampe! using the e-ink device "M5Paper".

It seems that noone has ever used Rust on it, so this is mostly about getting the Display working. As far as I remember, I was not yet successful.

## How to build on my MacBook
```
cd ~/esp/esp-idf
. ./export.sh
cd /Users/lena/bestelampe/software/bestefernbedienung
CRATE_CC_NO_DEFAULTS=1 cargo build
````
