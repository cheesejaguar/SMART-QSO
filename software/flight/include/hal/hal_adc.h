/**
 * @file hal_adc.h
 * @brief HAL ADC Interface
 *
 * Provides analog-to-digital converter abstraction for sensor readings.
 *
 * @requirement SRS-F070 Collect telemetry from all sensors
 */

#ifndef SMART_QSO_HAL_ADC_H
#define SMART_QSO_HAL_ADC_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* ADC Configuration                                                          */
/*===========================================================================*/

/**
 * @brief ADC channel enumeration
 */
typedef enum {
    ADC_CHANNEL_VBATT       = 0,   /**< Battery voltage */
    ADC_CHANNEL_IBATT       = 1,   /**< Battery current */
    ADC_CHANNEL_VSOLAR      = 2,   /**< Solar panel voltage */
    ADC_CHANNEL_ISOLAR      = 3,   /**< Solar panel current */
    ADC_CHANNEL_VBUS        = 4,   /**< 5V bus voltage */
    ADC_CHANNEL_TEMP_MCU    = 5,   /**< MCU temperature */
    ADC_CHANNEL_TEMP_BOARD  = 6,   /**< Board temperature */
    ADC_CHANNEL_COUNT
} HalAdcChannel_t;

/**
 * @brief ADC resolution
 */
typedef enum {
    ADC_RESOLUTION_8BIT  = 8,
    ADC_RESOLUTION_10BIT = 10,
    ADC_RESOLUTION_12BIT = 12,
    ADC_RESOLUTION_14BIT = 14,
    ADC_RESOLUTION_16BIT = 16
} HalAdcResolution_t;

/**
 * @brief ADC reference voltage
 */
typedef enum {
    ADC_REF_INTERNAL = 0,   /**< Internal reference (typically 1.2-3.3V) */
    ADC_REF_EXTERNAL = 1,   /**< External reference */
    ADC_REF_VDD      = 2    /**< VDD as reference */
} HalAdcRef_t;

/*===========================================================================*/
/* ADC Functions                                                              */
/*===========================================================================*/

/**
 * @brief Initialize ADC subsystem
 *
 * @param resolution ADC resolution
 * @param reference  Voltage reference
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_adc_init(HalAdcResolution_t resolution, HalAdcRef_t reference);

/**
 * @brief Read raw ADC value
 *
 * @param channel ADC channel
 * @param value   Output: raw ADC value
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_adc_read_raw(HalAdcChannel_t channel, uint16_t *value);

/**
 * @brief Read ADC as voltage
 *
 * @param channel ADC channel
 * @param voltage Output: voltage in volts
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_adc_read_voltage(HalAdcChannel_t channel, double *voltage);

/**
 * @brief Read multiple channels
 *
 * @param channels Array of channels to read
 * @param values   Output: array of raw values
 * @param count    Number of channels
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_adc_read_multiple(const HalAdcChannel_t *channels,
                                        uint16_t *values, size_t count);

/**
 * @brief Calibrate ADC
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_adc_calibrate(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_ADC_H */
