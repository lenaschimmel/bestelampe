<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## LED Module E Adaptor
This was a quick and easy design, nothing fancy. The simple 2-layer board makes it easy to connect an ESP32-C6-DevKit-C1 to the LED Module E, and hook up a power source to `VLED`.

It also adds two I2C ports as Grove connectors, one with 5V and the other one with 3.3V supply voltage. **Be careful with 5V though: This board has no level shifter and the ESP32 can't take 5V. Only use this port if you know what you are doing.**

### Connector J4
The four pads of `J4` are hooked up to VLED on the adaptor board, but there are nor corresponding pins on the LED Module E. You might wonder: Why?

It's currently hard for me to find cable connectors with 2x11 pins. I could easily buy connectors with 2x13 pins though. For the through hole pads on the adaptor board, it's mandatory to have some extra holes to solder in the larger connector. On the LED modules, I use an SMD connector and don't have this problem.

Connecting the four extra pins to `VLED` is just future-proofing for later LED modules which might actually have a 2x13 connector.

### Revision 1.1
In the initial v1.0, the GND copper planes were very fragmented on both layers. If you would use the screw terminal `J5` Adaptor to supply `VLED`, the return current might be rather suboptimal.

So in v1.1, many tracks are re-routed to make better connections between `J5` and the GND pins on `J3`.

The extra pads for 3.3V and 5V have also swapped their positions (and are correctly labeled in both revisions).
