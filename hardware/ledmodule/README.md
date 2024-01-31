## LED module(s)
The besteLampe! v1 introduces a 14-pin connector for LED modules. 

A very basic module would only have to use two of those Pins: One of the six PWM channel pins, and the 24V pin (5V would also work, but is less efficient in most cases).

More complex modules may have up to six color channels, and may contain one more temperature sensors. They can also contain addressable LEDs, though that is not recommended for the main lighting.

### Current state
There is no PCB design yet. I manually produced a simple prototype using short LED strips, but 

### Future changes
 * Might switch to a more compact connector
 * Should have more pins for +24V and less for GND
 * Maybe it would be better to have the Mosfets on the LED module. Then they could be sized appropriately for the number and power of the LEDs on the module. On the other hand, it should be possible to just chain larger Mosfets on a LED module to the small ones on the main module.