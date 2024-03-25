## LED Module C
This module is very different from the previous ones. It can carry up to 25 small, high-powered *Cree XLamp Element* LEDs within a circle of just 17.5mm (of which the LEDs actually take up only 15.3mm). It is designed to fit a *HEKLA mounting adaptor* from *LEDiL*, which can carry many different reflectors or lenses from that manufacturer.

**Note: This module cannot be attached directly to the Main Module v1. It needs a driver and/or adaptor that is not yet designed. See the section "Driver board" below.**

<img src="/assets/module_c_3d.jpg" width="66%"/>

### Color mixing
Some of the LEDiL lenses officially support color mixing, by placing multiple LEDs with different colors behind. But to obtain a unifom color, it is needed to optimize the placement of the LEDs.

This testing module leads both anode and cathode of all 25 LEDs to the circumference, so that any combination of LEDs can be put in series by some jumper wires. For a final production design, the LED series would be known in advance and hardwired internally without pin headers and jumper wires.

### Thermal management
Some XLamp Element LEDs can be powered with up to 5 Watts **each**, so a fully equiped LED Module C could generate about 125 Watts of heat. It is not designed for such high power operation. Instead, I hope that it can be used with up to 15 Watts continously, only using a small subset of the 25 LEDs at once.

Even with *only* 15 Watts, a heat sink will be needed - the board dimensions and its thermal conductivity will probably not suffice to radiate the heat into the surrounding air. The PCB should spread the heat accross the circular ground plane on the bottom, so a heat sink should ideally cover that area. There are circular heat sinks - if you don't have one, chose a square one that covers as much of the bottom circle as possible without touching the surrounding circle of pin headers.

Like all LED Modules in this repository, it features a temperature sensor. It's the first one to switch completely to a TI TMP1075 with I2C, which allows 32 addresses to be selected via the solder jumpers on the board. The small *8-WSON*-package was chosen because of its thermal pad and to fit it within the center of the HEKLA mounting adaptor, so that the temperature will be sensed near the LEDs.

### Driver board
Because of the high power of each single LED, it is not practical to put 6 to 8 LEDs in series like I did on the LED Modules A and A2. Instead, there could be 1 to 3 LEDs in series, or in some cases 4 or 5.

As such, it's no longer feasible to use a fixed voltage of 24 V with simple PWM and a resistor in each LED series.

The preferred way to drive this board is with one PWM-dimmable constant current LED driver per LED color series, and without a current-limiting resistor at all. (But: all constant current LED drivers that I know need a very small sensing resistor in series, which has almost no current drop or power loss.)

When choosing constant current LED drivers, you have to choose two of these:
 - more than one channel
 - significantly more than 100mA per channel (these LEDs need between 1000mA and 3000mA)
 - generates its own PWM signal

Thus my plan is to use a TLC59711 as a first stage, which can generate fast 16 Bit PWM signals for 12 independant channels, but cannot provide much power. As a second stage, each channel should get a TPS92200, which takes a PWM signal and mulates it ontop of its high-power constant current drive.

As soon as the first LED Module C is built, I will test it with the TLC59711 and a single TPS92200D1EVM evaluation kit. 

If this works as expected, I will design a custom driver board that combines a single TLC59711 with 6 to 12 TPS92200 chips and attaches to one LED Module C.

And if that works, a later iteration might combine both driver stages and the LEDs on a single board, as well as an EPROM to store the modules identification and calibration data.

### Inner layers
This is the first time a designed a PCB with more than two layers, so the intertwined conductors that attach all the LEDs are not visible from the outside - which is a pity, since I think they look quite interesting:

<img src="/assets/module_c_layers.jpg" width="66%"/>

### Schematic mismatch
Watch out, the part references in the Schematic do not match the ones in the PCB layout, so "Update PCB from Schematic" is not possible at the moment! This will be fixed later.