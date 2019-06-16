/* Hello World Example

   This example code is in the Public Domain (or CC0 licensed, at your option.)

   Unless required by applicable law or agreed to in writing, this
   software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
   CONDITIONS OF ANY KIND, either express or implied.
*/
#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"
#include "esp_spi_flash.h"
#include <esp_wifi.h>

#include <Arduino.h>
#include <M5Stack.h>

extern const wifi_init_config_t esp_wifi_init_config_default = WIFI_INIT_CONFIG_DEFAULT();

extern "C" void rust_main();
extern "C" {
    void lcd_print(const char* s, std::size_t count)
    {
        for( std::size_t i = 0; i < count; i++ ) {
            M5.Lcd.print(*s++);
        }
    }
    void m5display_drawLine(int32_t x0, int32_t y0, int32_t x1, int32_t y1, uint32_t color) {
        M5.Lcd.drawLine(x0, y0, x1, y1, color);
    }
}

void loopTask(void*)
{
    printf("Starting loop task.\n");
    fflush(stdout);

    //M5.begin(true, true, false, false);

    rust_main();
    fflush(stdout);
    esp_restart();
}

extern "C" void app_main()
{
    //initArduino();
    xTaskCreatePinnedToCore(loopTask, "loopTask", 8192, NULL, 1, NULL, 1);
}
