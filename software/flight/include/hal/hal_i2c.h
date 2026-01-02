/**
 * @file hal_i2c.h
 * @brief HAL I2C Interface
 *
 * Provides I2C bus abstraction for EPS and sensor communication.
 *
 * @requirement SRS-I010 Software shall interface with EPS via I2C
 */

#ifndef SMART_QSO_HAL_I2C_H
#define SMART_QSO_HAL_I2C_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* I2C Configuration                                                          */
/*===========================================================================*/

/**
 * @brief I2C bus enumeration
 */
typedef enum {
    I2C_BUS_0 = 0,   /**< Primary I2C bus (EPS, sensors) */
    I2C_BUS_1 = 1,   /**< Secondary I2C bus (if available) */
    I2C_BUS_COUNT
} HalI2cBus_t;

/**
 * @brief I2C clock speed
 */
typedef enum {
    I2C_SPEED_100K  = 100000,    /**< Standard mode */
    I2C_SPEED_400K  = 400000,    /**< Fast mode */
    I2C_SPEED_1M    = 1000000    /**< Fast mode plus */
} HalI2cSpeed_t;

/**
 * @brief Known I2C device addresses
 */
#define I2C_ADDR_EPS_CONTROLLER   0x48
#define I2C_ADDR_BATTERY_MONITOR  0x55
#define I2C_ADDR_SOLAR_MONITOR    0x40
#define I2C_ADDR_TEMP_SENSOR_1    0x48
#define I2C_ADDR_TEMP_SENSOR_2    0x49

/*===========================================================================*/
/* I2C Functions                                                              */
/*===========================================================================*/

/**
 * @brief Initialize I2C bus
 *
 * @param bus   I2C bus to initialize
 * @param speed Clock speed
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_i2c_init(HalI2cBus_t bus, HalI2cSpeed_t speed);

/**
 * @brief Write data to I2C device
 *
 * @param bus     I2C bus
 * @param addr    7-bit device address
 * @param data    Data to write
 * @param len     Number of bytes to write
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_i2c_write(HalI2cBus_t bus, uint8_t addr,
                                const uint8_t *data, size_t len);

/**
 * @brief Read data from I2C device
 *
 * @param bus     I2C bus
 * @param addr    7-bit device address
 * @param data    Buffer to store read data
 * @param len     Number of bytes to read
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_i2c_read(HalI2cBus_t bus, uint8_t addr,
                               uint8_t *data, size_t len);

/**
 * @brief Write then read (typical register access pattern)
 *
 * @param bus       I2C bus
 * @param addr      7-bit device address
 * @param write_data Data to write (typically register address)
 * @param write_len  Number of bytes to write
 * @param read_data  Buffer to store read data
 * @param read_len   Number of bytes to read
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_i2c_write_read(HalI2cBus_t bus, uint8_t addr,
                                     const uint8_t *write_data, size_t write_len,
                                     uint8_t *read_data, size_t read_len);

/**
 * @brief Check if device is present on bus
 *
 * @param bus  I2C bus
 * @param addr 7-bit device address
 * @return true if device responds to address
 */
bool hal_i2c_device_present(HalI2cBus_t bus, uint8_t addr);

/**
 * @brief Perform I2C bus recovery (9 clock pulses)
 *
 * @param bus I2C bus
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_i2c_recover(HalI2cBus_t bus);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_I2C_H */
