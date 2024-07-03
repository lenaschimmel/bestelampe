<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## LED Module E
This LED module features 10 small, high-powered Cree XLamp Element LEDs on its short edge, and some of the electronics needed to power them.

### Important parts
 - CD74HCT244 as an 8-channel driver for the MOSFETs
 - optional pull-down resistors for the inputs and/or outputs of the CD74HCT244
 - MOSFETs
 - large series resistors
 - temperature sensor
 - small EPROM for identification

### Improvements from Module D
This board is partly based on the failed [LED Module D](../LED_Module_D/), but with a lot of changes:

 - larger board
 - re-assinged all components to the three power rails
 - removed the TLC59711 as PWM generator
 - instead 8 separate inputs for PWM signals
 - two of the input signals are duplicated to power 2 LEDs each (but with separate MOSFETs and resistors)
 - more and larger mounting holes
 - improved thermal conductance (more thermal vias, etc.)
 - optimized traces
 - optional pull-ups for the I2C bus
 - solder jumper to directly tie the alert signal of the temperature sensor to the enable input of the driver
 - added test pads
 - added bulk capacitor
 - switched to 100% SMD parts
 - LED voltage can be supplied via USB-C or via two pins of the Pin header (see below)

The placement of the LEDs is still the same as on Board D, which proved to be a good fit for the linear half-bown reflector prototype.

### LED supply voltage
With single LEDs (parallel, in contrast to series / strings), a voltage around 4V would be a good choice for the supply voltage, so that the series resistors don't waste too much energy.

But it's surprisingly difficult to find a good source for 4V if you've got only 230V AC, 5V and 3.3V DC to work with. It seems much more straight-forward to use 5V and adjust the series resistors to it.

The CD74HCT244 needs 4.5 V to 5.5 V to operate, so it already had a 5V rail. The power rail for the LEDs was historically called 4V, and even though it's lablled `VLED` on the silkscreen now, the nets are still called 4V in Kicad. Both rails are still separate, just in case I'll find a good 4V source in the near future.

Originally, this module should have a barrel jack connector to allow larger currents than the pin headers could. With 5V as the likely default voltage, it was obvious to change it into a USB-C connector, since USB-C should always be able to provide 5V at 3A if the CC lines have the correct resistors attached. To drive the board with 4V (or anything that's not 5V) you would need to choose one of these options:

 - somehow get your custom voltage onto a USB-C plug
 - use pin 21 and 22 on the pin header
 - use the through-hole test pad `VLED` (which was still named `+4V` on my first production run)