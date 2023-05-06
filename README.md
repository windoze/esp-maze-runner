# A Maze Runner on ESP32 with HX8369 TFT display and GT911 touch screen

HX8369 TFT driver was grabbed from the demo source code came with the board with slight modifications, as it was originally written for LVGL. Some wrappers were added to make it work with `embedded-graphics` crate. It may work on other boards with HX8369 display with correct parameters set in `hx8369.h`, but it's not tested.

The GT911 driver is not fully functional, it doesn't support multi-touch and the way it's being used may be completely wrong.