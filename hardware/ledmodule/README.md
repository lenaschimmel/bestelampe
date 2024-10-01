<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## v1 LED modules
The besteLampe! main module v1 introduced a 14-pin connector for LED modules. 

A very basic module would only have to use two of those Pins: One of the six PWM channel pins, and the 24V pin (5V would also work, but is less efficient in most cases).

More complex modules may have up to six color channels, and may contain one more temperature sensors. They can also contain addressable LEDs, though that is not recommended for the main lighting.

 * Module A and A2 use 6 to 21 mid-power LEDs for each of its six color channels. They are driven with 24V and current-limmiting resistors, so they are directly compatible with Main Module v1.
 * Module B is / was a concept for a slim, quarter-circle module. It's not clear if it will be continued, or made obsolete by Module C.

## Post-v1 modules
Later I experimented a lot with different LEDs, reflectors, lenses, PWM generators, constant-current drivers, etc. These experiments cannot be directly connected to a v1 main module, since they don't have a compatible 14-pin connector.

 * Module C is a testing module for a novel approach with 1 to 3 high-power LEDs per channel, and needs to be driven by a PWM-capable constand-current driver that does not yet exist. Compatibility with Main Module v1 is still unclear. It was mostly tested with low currents (60mA per LED) and worked very vell, but high-current testing is still needed.
 * Module D is a failed attempt. Do not use.
 * Module E is a heavily modified version of Module D, which (hopefully) works better.
     * Module E adaptor, which lets you connect an ESP32-C6-DevKit-C1 to the Module E V1.0 easily
     * Module E was moved to [ABL](../abl/LED_Module_E/) when it was bumped to Version 2.0 and is no longer compatible with the Adaptor.

It's still unclear how the future connector between a main module and LED module(s) will look like...