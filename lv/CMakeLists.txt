set(srcs "i2c_manager.c" "gt911.c" "hx8369.c")

idf_component_register(SRCS ${srcs}
                       INCLUDE_DIRS ${includes_public} "."
                       PRIV_INCLUDE_DIRS ${includes_private}
                       PRIV_REQUIRES "esp_lcd"
                       )
