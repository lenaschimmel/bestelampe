<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## LED Module D
### DO NOT USE
This module is flawed in countless ways, and I really recommend to not use it in any way. Not as it is, not as a basis for modification.

I just uploaded it for transparency, because I talk a lot about it on the [Progress](https://github.com/lenaschimmel/bestelampe/wiki/Progress) page. You can read about it's problems there.

### Goal
This board should make it possible to have up to 10 high power LEDs with different colors aranged in a slim (2mm) and short (20mm) line. A TLC59711 is used to generate the 10 PWM signals, but it is not driving the LEDs directly. Instead, a line driver, MOSFETs and series resistors are used to power the LEDs without the complexities of a constant current drive.