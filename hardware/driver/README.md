## Driver Modules
Some LEDs are better driven by constant current drivers (CCDs) than by simple MOSFETs and series resistors. See ["Driving LEDs" in the Wiki](https://github.com/lenaschimmel/bestelampe/wiki/Driving-LEDs) for more background on this.

There are multiple Chips which integrate much of the complexity of such drivers, but all of those ICs need specific peripherals around them to actually work. If paired with the correct peripherals, they should work in a very similar way, so that they could be exchanged for one another.

For low-power CCDs like the TLC59711, breakout-boards are available. But for high-power CCDs, no such breakout-boards existed. There were only evaluation-boards, which are 10 times bigger and 10 times more expensive than a breakout-board.

In this directory, I try to add such breakout-boards with a common footprint and pin configuration:

- Driver Module X - an empty template for actual modules
- Driver Module A - for the Allegro A6211

Other drivers that might get a driver module:
- Texas Instruments TPS92200
- Diodes Inc. AL8860
- _If you know similar drivers, let me know!_

I've looked at theses ones as well, but it's unlikely I will make a driver module for them:
- WLMDU9456001JT / 172946001 / MagI3C Power Module (only 450mA and very expesive)
- STM ALED600 (rather expensive and needs many peripherals)