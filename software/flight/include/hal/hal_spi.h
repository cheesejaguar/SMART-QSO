/**
 * @file hal_spi.h
 * @brief HAL SPI Interface
 *
 * SPI buses of the OBC (SQSO-ICX-001 B §4.3): the ADCS gyros. A bus is one
 * device: SPI2 carries two (the BMI088's gyro and accelerometer, each with
 * its own chip select), so it is two logical buses. The FRAM has its own
 * SPI1 behind hal_flash.h.
 *
 * @requirement SRS-F140 ADCS sensor drivers on the OBC buses
 */

#ifndef SMART_QSO_HAL_SPI_H
#define SMART_QSO_HAL_SPI_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* SPI Configuration                                                          */
/*===========================================================================*/

/**
 * @brief SPI bus enumeration
 */
typedef enum {
    SPI_BUS_0 = 0,   /**< SPI2, CS PD0: BMI088 gyro (cold-spare gyro), up to 10 MHz */
    SPI_BUS_1 = 1,   /**< SPI3, CS PD1: Epson M-G370PDT0 (primary gyro), mode 3, up to 1 MHz */
    SPI_BUS_2 = 2,   /**< SPI2, CS PD4: BMI088 accelerometer (shares SPI_BUS_0's clock and data lines) */
    SPI_BUS_COUNT
} HalSpiBus_t;

/**
 * @brief SPI mode (clock polarity and phase)
 */
typedef enum {
    SPI_MODE_0 = 0,  /**< CPOL=0, CPHA=0 */
    SPI_MODE_1 = 1,  /**< CPOL=0, CPHA=1 */
    SPI_MODE_2 = 2,  /**< CPOL=1, CPHA=0 */
    SPI_MODE_3 = 3   /**< CPOL=1, CPHA=1 */
} HalSpiMode_t;

/**
 * @brief SPI configuration structure
 */
typedef struct {
    uint32_t clock_hz;     /**< Clock frequency in Hz */
    HalSpiMode_t mode;     /**< SPI mode */
    uint8_t bits_per_word; /**< Typically 8 */
    bool msb_first;        /**< true for MSB first */
} HalSpiConfig_t;

/*===========================================================================*/
/* SPI Functions                                                              */
/*===========================================================================*/

/**
 * @brief Initialize SPI bus
 *
 * @param bus    SPI bus to initialize
 * @param config Configuration parameters
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_spi_init(HalSpiBus_t bus, const HalSpiConfig_t *config);

/**
 * @brief Transfer data on SPI (full duplex)
 *
 * @param bus     SPI bus
 * @param tx_data Data to transmit (can be NULL for receive only)
 * @param rx_data Buffer for received data (can be NULL for transmit only)
 * @param len     Number of bytes to transfer
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_spi_transfer(HalSpiBus_t bus, const uint8_t *tx_data,
                                   uint8_t *rx_data, size_t len);

/**
 * @brief Write data on SPI (transmit only)
 *
 * @param bus  SPI bus
 * @param data Data to transmit
 * @param len  Number of bytes to transmit
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_spi_write(HalSpiBus_t bus, const uint8_t *data, size_t len);

/**
 * @brief Read data on SPI (receive only)
 *
 * @param bus  SPI bus
 * @param data Buffer for received data
 * @param len  Number of bytes to receive
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_spi_read(HalSpiBus_t bus, uint8_t *data, size_t len);

/**
 * @brief Assert chip select (active low)
 *
 * @param bus SPI bus
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_spi_cs_assert(HalSpiBus_t bus);

/**
 * @brief De-assert chip select
 *
 * @param bus SPI bus
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_spi_cs_deassert(HalSpiBus_t bus);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_SPI_H */
