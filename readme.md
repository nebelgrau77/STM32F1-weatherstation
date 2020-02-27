Simple weather station using STM32F103 "blue pill" board,
Bosch BME280 sensor and SSD1306 OLED display.

It uses the shared-bus crate to share the I2C bus between the sensor and the display.

At the moment I don't know how or whether it is possible to port this to STM32F0xx or STM32F4xx boards:
the BME280 crate requires the BlockingI2c trait, which is not implemented for those crates.

The update occurs every n cycles of the processor, delay provided with cortex_m::asm::delay
This is due to the fact that the BME280 driver consumes the delay instance: another possibility of a future improvement.

The great "need to reset the board after powering it up to get the display going" mystery remains unsolved :)