//! The HAL's logical pins, ADC channels and buses (`hal/hal_gpio.h`,
//! `hal/hal_adc.h`, `hal/hal_spi.h`, `hal/hal_i2c.h`, `hal/hal_capture.h`)
//! added with the OBC board's pin allocation (SQSO-ICX-001 B §6), as the
//! flight software uses them. The pins of the earlier headers stay with
//! their modules (`rf_control`, `deployment`, ...). `tests/test_hal_pins.c`
//! compares every value here with the C headers.
#![allow(dead_code)] // the target-only drivers use some of them

/// `GPIO_PIN_PAYLOAD_EN`
pub(crate) const PAYLOAD_EN: u32 = 0;
/// `GPIO_PIN_RADIO_EN`
pub(crate) const RADIO_EN: u32 = 1;
/// `GPIO_PIN_JETSON_PWR`
pub(crate) const JETSON_PWR: u32 = 6;
/// `GPIO_PIN_GNSS_EN`
pub(crate) const GNSS_EN: u32 = 17;
/// `GPIO_PIN_GYRO_EN`
pub(crate) const GYRO_EN: u32 = 25;
/// `GPIO_PIN_IMU2_EN`
pub(crate) const IMU2_EN: u32 = 26;
/// `GPIO_PIN_SNS_EN`
pub(crate) const SNS_EN: u32 = 27;
/// `GPIO_PIN_GYRO_RB`
pub(crate) const GYRO_RB: u32 = 28;
/// `GPIO_PIN_IMU2_RB`
pub(crate) const IMU2_RB: u32 = 29;
/// `GPIO_PIN_SNS_RB`
pub(crate) const SNS_RB: u32 = 30;
/// `GPIO_PIN_GYRO_RST_N`
pub(crate) const GYRO_RST_N: u32 = 31;
/// `GPIO_PIN_GYRO_DRDY`
pub(crate) const GYRO_DRDY: u32 = 32;
/// `GPIO_PIN_IMU2_DRDY`
pub(crate) const IMU2_DRDY: u32 = 33;
/// `GPIO_PIN_MAG_DRDY`
pub(crate) const MAG_DRDY: u32 = 34;
/// `GPIO_PIN_MTQ_X_DIR` (Y and Z follow)
pub(crate) const MTQ_DIR: [u32; 3] = [35, 36, 37];
/// `GPIO_PIN_MTQ_FAULT_N`
pub(crate) const MTQ_FAULT_N: u32 = 38;
/// `GPIO_PIN_MTQ_EXPIRED`
pub(crate) const MTQ_EXPIRED: u32 = 39;
/// `GPIO_PIN_TONE_OK`
pub(crate) const TONE_OK: u32 = 40;
/// `GPIO_PIN_XPDR_KEY`
pub(crate) const XPDR_KEY: u32 = 41;
/// `GPIO_PIN_PTT_EXC`
pub(crate) const PTT_EXC: u32 = 42;
/// `GPIO_PIN_JET_PG`
pub(crate) const JET_PG: u32 = 43;
/// `GPIO_PIN_GNSS_PG`
pub(crate) const GNSS_PG: u32 = 44;
/// `GPIO_PIN_WDI`
pub(crate) const WDI: u32 = 45;
/// `GPIO_PIN_MAG_EN`: the magnetometer's switch U35 (PF14)
pub(crate) const MAG_EN: u32 = 46;
/// `GPIO_PIN_MAG_RB`: its EN-node readback (PF15)
pub(crate) const MAG_RB: u32 = 47;
/// `GPIO_PIN_COUNT`
pub(crate) const GPIO_PIN_COUNT: u32 = 48;

/// `ADC_CHANNEL_VBATT`
pub(crate) const ADC_VBATT: u32 = 0;
/// `ADC_CHANNEL_TEMP_BOARD`
pub(crate) const ADC_TEMP_BOARD: u32 = 6;
/// `ADC_CHANNEL_VSW1_RB`
pub(crate) const ADC_VSW1_RB: u32 = 10;
/// `ADC_CHANNEL_PT_NTC`
pub(crate) const ADC_PT_NTC: u32 = 11;
/// `ADC_CHANNEL_MTQ_X_I` (Y and Z follow)
pub(crate) const ADC_MTQ_I: [u32; 3] = [12, 13, 14];
/// `ADC_CHANNEL_VJET_RB`
pub(crate) const ADC_VJET_RB: u32 = 15;
/// `ADC_CHANNEL_COUNT`
pub(crate) const ADC_CHANNEL_COUNT: u32 = 16;

/// `SPI_BUS_0`: the BMI088 gyro
pub(crate) const SPI_BMI088_GYRO: u32 = 0;
/// `SPI_BUS_1`: the M-G370PDT0
pub(crate) const SPI_MG370: u32 = 1;
/// `SPI_BUS_2`: the BMI088 accelerometer
pub(crate) const SPI_BMI088_ACCEL: u32 = 2;
/// `SPI_BUS_COUNT`
pub(crate) const SPI_BUS_COUNT: u32 = 3;

/// `I2C_BUS_0`: EPS controller, INA228s, AntS
pub(crate) const I2C_EPS: u32 = 0;
/// `I2C_BUS_1`: ADS7138 and RM3100
pub(crate) const I2C_ADCS: u32 = 1;
/// `I2C_BUS_2`: the RV-3028-C7
pub(crate) const I2C_RTC: u32 = 2;
/// `I2C_BUS_3`: the AntS alone (I2C3)
pub(crate) const I2C_ANTS: u32 = 3;
/// `I2C_BUS_COUNT`
pub(crate) const I2C_BUS_COUNT: u32 = 4;

/// `FLASH_REGION_ADCS_CAL`
pub(crate) const FLASH_REGION_ADCS_CAL: u32 = 16;
/// `FLASH_REGION_COUNT`
pub(crate) const FLASH_REGION_COUNT: u32 = 17;

/// `CAPTURE_GNSS_PPS`
pub(crate) const CAPTURE_GNSS_PPS: u32 = 0;
/// `CAPTURE_RTC_1HZ`
pub(crate) const CAPTURE_RTC_1HZ: u32 = 1;
/// `CAPTURE_PTT_EXC`
pub(crate) const CAPTURE_PTT_EXC: u32 = 2;
