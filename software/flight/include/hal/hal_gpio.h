/**
 * @file hal_gpio.h
 * @brief HAL GPIO Interface
 *
 * Provides GPIO control abstraction for load switches and signals.
 *
 * @requirement SRS-F011 Control power to subsystems via load switches
 */

#ifndef SMART_QSO_HAL_GPIO_H
#define SMART_QSO_HAL_GPIO_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* GPIO Pin Definitions                                                       */
/*===========================================================================*/

/**
 * @brief GPIO pin enumeration
 *
 * Logical pin names; the STM32L4R5 pin of each is the master allocation
 * SQSO-ICX-001 B §4.1 (hardware/electrical/INTERCONNECT.md), in
 * software/rust/firmware/src/gpio.rs.
 */
typedef enum {
    GPIO_PIN_PAYLOAD_EN    = 0,   /**< PB2: Jetson payload switch SW1 (PAYLOAD_EN) */
    GPIO_PIN_RADIO_EN      = 1,   /**< PB5: radio power enable (SW2) */
    GPIO_PIN_ADCS_EN       = 2,   /**< PB6: ADCS power enable (SW4 and the coil supply SW11) */
    GPIO_PIN_HEATER_EN     = 3,   /**< PD2: battery heater (SW5) */
    GPIO_PIN_DEPLOY_EN     = 4,   /**< PF3: deployment master arm (a burn needs this and the element pin) */
    GPIO_PIN_LED_STATUS    = 5,   /**< PD3: status LED */
    GPIO_PIN_JETSON_PWR    = 6,   /**< PA8: Jetson eFuse U19 (JETSON_PWR) */
    GPIO_PIN_RF_TX_EN      = 7,   /**< PA3: transmitter PA rail (RF_TX_EN; low = TX-OFF) */
    GPIO_PIN_ALERT         = 8,   /**< PB7: EPS alert input (open drain, active low) */
    GPIO_PIN_DEPLOY_SENSE  = 9,   /**< PE2: separation switch DS2 sense input: high = released (EPS 1 MΩ source), no internal pull, separation_switch.h */
    GPIO_PIN_UMBILICAL_SENSE = 10, /**< PB8: GSE umbilical present (high); pulled low in flight */
    GPIO_PIN_ANTENNA_BURN  = 11,  /**< Wing 2 resistor A (PB9; was the antenna burn: the AntS deploys over I2C) */
    GPIO_PIN_SOLAR_BURN    = 12,  /**< Wing 1 resistor A (PB10) */
    GPIO_PIN_ANTENNA_SENSE = 13,  /**< Not wired since SQSO-ICX-001 B (the AntS reports its elements over I2C): reads low */
    GPIO_PIN_SOLAR_SENSE   = 14,  /**< PB12: solar wings deployed sense input (low = both deployed) */
    GPIO_PIN_TX_PWR_SEL    = 15,  /**< PE3: downlink power select: low 0.5 W, high 1 W */
    GPIO_PIN_XPDR_EN       = 16,  /**< PE4: FM transponder enable: heartbeat (hal_gpio_heartbeat), never a level */
    GPIO_PIN_GNSS_EN       = 17,  /**< PE5: GNSS receiver load switch SW7 (current-limited, latch-up trip) */
    GPIO_PIN_SOLAR_BURN_B  = 18,  /**< Wing 1 resistor B (PE6) */
    /* Logical pins added at CDR (STM32 pins: SQSO-ICX-001 B §4.1) */
    GPIO_PIN_EOL_ARM       = 19,  /**< PF4: EPS end-of-life charge disconnect: arm latch (rising edge sets it), passivation.h */
    GPIO_PIN_EOL_FIRE      = 20,  /**< PG1: EPS end-of-life charge disconnect: fire (level, in series with the latch) */
    GPIO_PIN_RX_CD         = 21,  /**< PG0, input: command receiver carrier detect (high on a carrier), receiver.h */
    GPIO_PIN_RX_FAULT      = 22,  /**< PF13, input: command receiver health, high on a fault (open reads healthy), receiver.h */
    GPIO_PIN_BURN_W2B      = 23,  /**< PE7: wing 2 resistor B */
    GPIO_PIN_LOCK_CLR      = 24,  /**< PF5: EPS recharge-lockout clear pulse (with an I2C unlock), eps_lockout.h */
    /* ADCS sensors and magnetorquers (SQSO-ICX-001 B §6 rows 11, 12, 16, 17) */
    GPIO_PIN_GYRO_EN       = 25,  /**< PF6: 3V3_GYRO latch-up switch (M-G370PDT0) */
    GPIO_PIN_IMU2_EN       = 26,  /**< PF7: 3V3_IMU2 latch-up switch (BMI088 cold spare) */
    GPIO_PIN_SNS_EN        = 27,  /**< PF8: 3V3_SNS latch-up switch (RM3100, ADS7138) */
    GPIO_PIN_GYRO_RB       = 28,  /**< PF10, input: gyro switch EN node; low with GYRO_EN high = trip */
    GPIO_PIN_IMU2_RB       = 29,  /**< PF11, input: BMI088 switch EN node */
    GPIO_PIN_SNS_RB        = 30,  /**< PF12, input: sensor switch EN node */
    GPIO_PIN_GYRO_RST_N    = 31,  /**< PD14: M-G370 /RST (low = reset, at least 100 ms) */
    GPIO_PIN_GYRO_DRDY     = 32,  /**< PD10, input: M-G370 data ready */
    GPIO_PIN_IMU2_DRDY     = 33,  /**< PD15, input: BMI088 gyro data ready (INT3) */
    GPIO_PIN_MAG_DRDY      = 34,  /**< PD11, input: RM3100 data ready */
    GPIO_PIN_MTQ_X_DIR     = 35,  /**< PE8: X bridge PH (direction) */
    GPIO_PIN_MTQ_Y_DIR     = 36,  /**< PE10: Y bridge PH */
    GPIO_PIN_MTQ_Z_DIR     = 37,  /**< PE12: Z bridge PH */
    GPIO_PIN_MTQ_FAULT_N   = 38,  /**< PE14, input: bridge nFAULT (wired-OR, low = fault) */
    GPIO_PIN_MTQ_EXPIRED   = 39,  /**< PE15, input: the hardware off-timer expired (high = the firmware's timer failed) */
    /* Readbacks (SQSO-ICX-001 B §6 rows 18-22) */
    GPIO_PIN_TONE_OK       = 40,  /**< PC9, input: CTCSS tone decoded (repeat path) */
    GPIO_PIN_XPDR_KEY      = 41,  /**< PG12, input: transponder repeat keyed */
    GPIO_PIN_PTT_EXC       = 42,  /**< PC7, input: exciter keyed (PTT_EXCITER; its keyed time: hal_capture.h) */
    GPIO_PIN_JET_PG        = 43,  /**< PG6, input: Jetson carrier 3.3 V up (JET_3V3) */
    GPIO_PIN_GNSS_PG       = 44,  /**< PG15, input: 3V3_GNSS up (low with GNSS_EN high = SW7 trip) */
    GPIO_PIN_WDI           = 45,  /**< PG11: TPS3850 supervisor kick (falling edge every 10 s) */
    GPIO_PIN_MAG_EN        = 46,  /**< PF14: magnetometer switch U35 (3V3_MAG), high = on (SQSO-ICX-001 D §6 row 33) */
    GPIO_PIN_MAG_RB        = 47,  /**< PF15, input: U35 EN node (low with MAG_EN high = latch-up trip) */
    GPIO_PIN_COUNT         = 48
} HalGpioPin_t;

/** The four wing burn lines, one resistor at a time (deployment.h) */
#define GPIO_PIN_BURN_W1A GPIO_PIN_SOLAR_BURN
#define GPIO_PIN_BURN_W1B GPIO_PIN_SOLAR_BURN_B
#define GPIO_PIN_BURN_W2A GPIO_PIN_ANTENNA_BURN
/** PTT_EXCITER readback (SQSO-ICX-001 B) */
#define GPIO_PIN_PTT_EXCITER GPIO_PIN_PTT_EXC

/**
 * @brief GPIO direction
 */
typedef enum {
    GPIO_DIR_INPUT  = 0,
    GPIO_DIR_OUTPUT = 1
} HalGpioDir_t;

/**
 * @brief GPIO pull configuration
 */
typedef enum {
    GPIO_PULL_NONE = 0,
    GPIO_PULL_UP   = 1,
    GPIO_PULL_DOWN = 2
} HalGpioPull_t;

/*===========================================================================*/
/* GPIO Functions                                                             */
/*===========================================================================*/

/**
 * @brief Initialize GPIO subsystem
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_init(void);

/**
 * @brief Configure a GPIO pin
 *
 * @param pin       Pin to configure
 * @param direction Input or output
 * @param pull      Pull-up/down configuration
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_config(HalGpioPin_t pin, HalGpioDir_t direction,
                                  HalGpioPull_t pull);

/**
 * @brief Set GPIO output state
 *
 * @param pin   Pin to set
 * @param state true for high, false for low
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_set(HalGpioPin_t pin, bool state);

/**
 * @brief Get GPIO input state
 *
 * @param pin Pin to read
 * @return true if high, false if low
 */
bool hal_gpio_get(HalGpioPin_t pin);

/**
 * @brief Toggle GPIO output
 *
 * @param pin Pin to toggle
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_toggle(HalGpioPin_t pin);

/**
 * @brief Heartbeat frequency of hal_gpio_heartbeat(), Hz
 *
 * docs/hardware/BUILD_VS_BUY.md §3.2: the radio board's retriggerable
 * monostable (about 50 ms) holds the transponder enable only while edges
 * arrive at about this rate.
 */
#define HAL_GPIO_HEARTBEAT_HZ 100U

/**
 * @brief Run or stop the heartbeat on GPIO_PIN_XPDR_EN
 *
 * A hardware timer toggles the pin at HAL_GPIO_HEARTBEAT_HZ for @p lease_ms
 * from this call and then stops it with the pin low; each call before the
 * lease runs out restarts the lease, so the wave runs only while the caller
 * keeps renewing it. @p lease_ms = 0 stops the wave now (pin low). The wave
 * is stopped at reset. A level written with hal_gpio_set() is not a
 * heartbeat: the radio board treats a pin held high as disabled.
 *
 * @param pin      GPIO_PIN_XPDR_EN (the only heartbeat pin)
 * @param lease_ms How long the wave runs from now, ms (0: stop)
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID for another pin;
 *         SMART_QSO_ERROR before hal_gpio_init()
 */
SmartQsoResult_t hal_gpio_heartbeat(HalGpioPin_t pin, uint32_t lease_ms);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_GPIO_H */
