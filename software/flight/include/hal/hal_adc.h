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
    ADC_CHANNEL_VBATT       = 0,   /**< PC2 (IN3): battery voltage, V (pack / 6 divider) */
    ADC_CHANNEL_IBATT       = 1,   /**< PC3 (IN4): battery current, A */
    ADC_CHANNEL_VSOLAR      = 2,   /**< PC4 (IN13): solar panel voltage, V */
    ADC_CHANNEL_ISOLAR      = 3,   /**< PC5 (IN14): solar panel current, A */
    ADC_CHANNEL_VBUS        = 4,   /**< PB0 (IN15): 5V bus voltage, V */
    ADC_CHANNEL_TEMP_MCU    = 5,   /**< internal: MCU temperature, °C */
    ADC_CHANNEL_TEMP_BOARD  = 6,   /**< PB1 (IN16): OBC board temperature (TMP235), °C */
    ADC_CHANNEL_TX_TEMP     = 7,   /**< PA0 (IN5): downlink transmitter (PA) temperature, °C */
    ADC_CHANNEL_TX_FWD      = 8,   /**< PC0 (IN1): downlink forward power, W */
    ADC_CHANNEL_TX_REV      = 9,   /**< PC1 (IN2): downlink reflected power, W */
    ADC_CHANNEL_VSW1_RB     = 10,  /**< PA1 (IN6): Jetson switch SW1 output readback, V (VBAT_SW1 / 6 divider) */
    ADC_CHANNEL_PT_NTC      = 11,  /**< PA2 (IN7): Jetson spreader thermistor, pin V (ratiometric: hal_adc_read_raw gives N, R = 10.0 kΩ N / (4095 − N)) */
    ADC_CHANNEL_MTQ_X_I     = 12,  /**< PA4 (IN9): X coil current (IPROPI 10 mV/mA), A */
    ADC_CHANNEL_MTQ_Y_I     = 13,  /**< PA5 (IN10): Y coil current, A */
    ADC_CHANNEL_MTQ_Z_I     = 14,  /**< PA6 (IN11): Z coil current, A */
    ADC_CHANNEL_VJET_RB     = 15,  /**< PA7 (IN12): Jetson eFuse U19 output readback, V (VBAT_JET / 6 divider) */
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
