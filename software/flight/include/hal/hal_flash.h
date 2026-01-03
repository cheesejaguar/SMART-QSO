/**
 * @file hal_flash.h
 * @brief HAL Flash/NVM Interface
 *
 * Provides non-volatile memory abstraction for data persistence.
 *
 * @requirement SRS-F060 Persist mission data to non-volatile memory
 * @requirement SRS-F041 Maintain fault log in non-volatile memory
 */

#ifndef SMART_QSO_HAL_FLASH_H
#define SMART_QSO_HAL_FLASH_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* Flash Configuration                                                        */
/*===========================================================================*/

/**
 * @brief Flash sector size (typically 256 bytes to 4KB depending on device)
 */
#define HAL_FLASH_SECTOR_SIZE   256

/**
 * @brief Flash page size (for write operations)
 */
#define HAL_FLASH_PAGE_SIZE     64

/**
 * @brief NVM storage regions
 */
typedef enum {
    FLASH_REGION_MISSION_DATA   = 0,   /**< Mission data storage */
    FLASH_REGION_EPS_CONFIG     = 1,   /**< EPS configuration */
    FLASH_REGION_SENSOR_CONFIG  = 2,   /**< Sensor configuration */
    FLASH_REGION_FAULT_LOG      = 3,   /**< Fault log storage */
    FLASH_REGION_BACKUP         = 4,   /**< Backup storage */
    FLASH_REGION_STATE          = 5,   /**< System state persistence */
    FLASH_REGION_COUNT
} HalFlashRegion_t;

/*===========================================================================*/
/* Flash Functions                                                            */
/*===========================================================================*/

/**
 * @brief Initialize flash subsystem
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_flash_init(void);

/**
 * @brief Read data from flash
 *
 * @param region Flash region
 * @param offset Offset within region
 * @param data   Buffer to store read data
 * @param len    Number of bytes to read
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_flash_read(HalFlashRegion_t region, uint32_t offset,
                                 uint8_t *data, size_t len);

/**
 * @brief Write data to flash
 *
 * @param region Flash region
 * @param offset Offset within region
 * @param data   Data to write
 * @param len    Number of bytes to write
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_flash_write(HalFlashRegion_t region, uint32_t offset,
                                  const uint8_t *data, size_t len);

/**
 * @brief Erase flash region
 *
 * @param region Flash region to erase
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_flash_erase(HalFlashRegion_t region);

/**
 * @brief Get region size
 *
 * @param region Flash region
 * @return Size in bytes
 */
size_t hal_flash_region_size(HalFlashRegion_t region);

/**
 * @brief Get region base address (for direct access if supported)
 *
 * @param region Flash region
 * @return Base address
 */
uint32_t hal_flash_region_base(HalFlashRegion_t region);

/**
 * @brief Check if flash is busy
 *
 * @return true if flash operation in progress
 */
bool hal_flash_busy(void);

/**
 * @brief Get flash wear level (if supported)
 *
 * @param region Flash region
 * @return Estimated wear level (0-100%)
 */
uint8_t hal_flash_wear_level(HalFlashRegion_t region);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_FLASH_H */
