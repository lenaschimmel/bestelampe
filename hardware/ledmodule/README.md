## LED module(s)
The besteLampe! v1 introduces a 14-pin connector for LED modules. 

A very basic module would only have to use two of those Pins: One of the six PWM channel pins, and the 24V pin (5V would also work, but is less efficient in most cases).

More complex modules may have up to six color channels, and may contain one more temperature sensors. They can also contain addressable LEDs, though that is not recommended for the main lighting.

### Current state
There are three modules online:
 * Module A and A2 use 6 to 21 mid-power LEDs for each of its six color channels. They are driven with 24V and current-limmiting resistors, so they are directly compatible with Main Module v1.
 * Module C is a testing module for a novel approach with 1 to 3 high-power LEDs per channel, and needs to be driven by a PWM-capable constand-current driver that does not yet exist. Compatibility with Main Module v1 may or may not be added later.

### Future changes (party implemented or obsoleted by Module C and its concept)
 * Might switch to a more compact connector
 * Should have more pins for +24V and less for GND
 * Maybe it would be better to have the Mosfets on the LED module. Then they could be sized appropriately for the number and power of the LEDs on the module. On the other hand, it should be possible to just chain larger Mosfets on a LED module to the small ones on the main module.