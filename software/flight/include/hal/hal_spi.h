/**
 * @file hal_spi.h
 * @brief HAL SPI Interface
 *
 * Provides SPI bus abstraction for RF transceiver communication.
 *
 * @requirement SRS-I020 Software shall control beacon transmitter
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
    SPI_BUS_0 = 0,   /**< Primary SPI (RF transceiver) */
    SPI_BUS_1 = 1,   /**< Secondary SPI (if available) */
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
