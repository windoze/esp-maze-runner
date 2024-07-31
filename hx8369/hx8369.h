#ifndef HX8369_H
#define HX8369_H

#ifdef __cplusplus
extern "C" {
#endif

/*********************
 *      INCLUDES
 *********************/
#include <stdbool.h>

#include "esp_lcd_panel_interface.h"
#include "esp_lcd_types.h"

#include "sdkconfig.h"

#define LCD_H_RES 800
#define LCD_V_RES 480
/*********************
 *      DEFINES
 *********************/
#define PIN_NUM_DATA0 46
#define PIN_NUM_DATA1 3
#define PIN_NUM_DATA2 8
#define PIN_NUM_DATA3 18
#define PIN_NUM_DATA4 17
#define PIN_NUM_DATA5 16
#define PIN_NUM_DATA6 15
#define PIN_NUM_DATA7 7

#define PIN_NUM_PCLK 10
#define PIN_NUM_CS 12
#define PIN_NUM_DC 11
#define PIN_NUM_RST 9
#define PIN_NUM_BK_LIGHT 6

// Bit number used to represent command and parameter
#define LCD_CMD_BITS 8
#define LCD_PARAM_BITS 8

#define LCD_PIXEL_CLOCK_HZ (20 * 1000 * 1000)
// Supported alignment: 16, 32, 64. A higher alignment can enables higher burst transfer size, thus a higher i80 bus throughput.
#define PSRAM_DATA_ALIGNMENT 64

/**********************
 * GLOBAL PROTOTYPES
 **********************/
extern      esp_lcd_panel_handle_t panel_handle;
esp_lcd_panel_handle_t    hx8369_init(void);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /*HX8369_H*/