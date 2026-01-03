/**
 * @file deployment.c
 * @brief SMART-QSO Deployment Control Module Implementation
 *
 * Implements deployment sequencing for antenna and solar panel deployment
 * following CubeSat Design Specification requirements and NASA Ames
 * SmallSat heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat).
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#include "deployment.h"
#include "fault_mgmt.h"
#include "flight_log.h"
#include "smart_qso.h"
#include "hal/hal_gpio.h"
#include "hal/hal_timer.h"

#include <string.h>

/*===========================================================================*/
/* Private Constants                                                          */
/*===========================================================================*/

/** GPIO pin for deployment switch input (mapped to GPIO_PIN_DEPLOY_SENSE) */
#define DEPLOY_SWITCH_PIN       GPIO_PIN_DEPLOY_SENSE

/** GPIO pin for deployment mechanism */
#define DEPLOY_MECH_PIN         GPIO_PIN_DEPLOY_EN

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Deployment module state */
static DeploymentStatus_t s_deploy_status;

/** Module initialized flag */
static bool s_initialized = false;

/** State name lookup table */
static const char *s_state_names[DEPLOY_STATE_COUNT] = {
    "INHIBITED",
    "WAITING",
    "ANTENNA_PENDING",
    "ANTENNA_ACTIVE",
    "ANTENNA_VERIFY",
    "SOLAR_PENDING",
    "SOLAR_ACTIVE",
    "SOLAR_VERIFY",
    "COMPLETE",
    "FAILED"
};

/*===========================================================================*/
/* Private Function Prototypes                                                */
/*===========================================================================*/

static void enter_state(DeploymentState_t new_state, uint32_t current_time_ms);
static bool check_deployment_switch(void);
static bool check_element_deployed(DeploymentElement_t element);
static void actuate_deployment(DeploymentElement_t element, bool activate);
static uint32_t calculate_crc(const DeploymentStatus_t *status);

/*===========================================================================*/
/* Public Functions                                                           */
/*===========================================================================*/

bool deployment_init(void) {
    if (s_initialized) {
        return true;
    }

    /* Initialize state */
    memset(&s_deploy_status, 0, sizeof(s_deploy_status));
    s_deploy_status.state = DEPLOY_STATE_INHIBITED;
    s_deploy_status.deployment_enabled = true;  /* Default enabled */

    /* Configure GPIO pins for deployment */
    (void)hal_gpio_config(GPIO_PIN_DEPLOY_SENSE, GPIO_DIR_INPUT, GPIO_PULL_UP);
    (void)hal_gpio_config(GPIO_PIN_DEPLOY_EN, GPIO_DIR_OUTPUT, GPIO_PULL_NONE);

    /* Ensure deployment mechanism is off */
    (void)hal_gpio_set(GPIO_PIN_DEPLOY_EN, false);

    /* Try to load saved state */
    if (!deployment_load_state()) {
        LOG_INFO("DEPLOY", "No saved state, starting fresh");
    }

    s_initialized = true;
    LOG_INFO("DEPLOY", "Deployment module initialized");

    return true;
}

void deployment_process(uint32_t current_time_ms) {
    if (!s_initialized) {
        return;
    }

    uint32_t time_in_state = current_time_ms - s_deploy_status.state_entry_time_ms;
    DeploymentElementStatus_t *antenna = &s_deploy_status.elements[DEPLOY_ELEMENT_ANTENNA];
    DeploymentElementStatus_t *solar = &s_deploy_status.elements[DEPLOY_ELEMENT_SOLAR_PANEL];

    switch (s_deploy_status.state) {
        case DEPLOY_STATE_INHIBITED:
            /* Check deployment switch */
            if (check_deployment_switch()) {
                s_deploy_status.separation_detected = true;
                s_deploy_status.separation_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_WAITING, current_time_ms);
                LOG_INFO("DEPLOY", "Separation detected");
            }
            break;

        case DEPLOY_STATE_WAITING:
            /* Wait for deployment timer */
            if (time_in_state >= DEPLOYMENT_WAIT_TIME_MS) {
                if (s_deploy_status.deployment_enabled) {
                    enter_state(DEPLOY_STATE_ANTENNA_PENDING, current_time_ms);
                    LOG_INFO("DEPLOY", "Wait complete, starting antenna deploy");
                }
            }
            break;

        case DEPLOY_STATE_ANTENNA_PENDING:
            /* Check if already deployed */
            if (check_element_deployed(DEPLOY_ELEMENT_ANTENNA)) {
                antenna->deployed = true;
                antenna->deploy_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_SOLAR_PENDING, current_time_ms);
                LOG_INFO("DEPLOY", "Antenna already deployed");
            } else if (antenna->attempt_count < DEPLOYMENT_MAX_RETRIES) {
                /* Start deployment */
                actuate_deployment(DEPLOY_ELEMENT_ANTENNA, true);
                antenna->deploy_attempted = true;
                antenna->attempt_count++;
                s_deploy_status.deploy_start_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_ANTENNA_ACTIVE, current_time_ms);
                LOG_INFO("DEPLOY", "Antenna deploy attempt %u",
                         (unsigned)antenna->attempt_count);
            } else {
                /* Max retries exceeded */
                fault_log_add(FAULT_TYPE_DEPLOYMENT, FAULT_SEVERITY_ERROR,
                             "Antenna deployment failed", 0.0);
                enter_state(DEPLOY_STATE_FAILED, current_time_ms);
                LOG_ERROR("DEPLOY", "Antenna deploy failed after %u attempts",
                          (unsigned)DEPLOYMENT_MAX_RETRIES);
            }
            break;

        case DEPLOY_STATE_ANTENNA_ACTIVE:
            /* Monitor burn time */
            if (time_in_state >= ANTENNA_DEPLOY_BURN_TIME_MS) {
                actuate_deployment(DEPLOY_ELEMENT_ANTENNA, false);
                enter_state(DEPLOY_STATE_ANTENNA_VERIFY, current_time_ms);
            }
            break;

        case DEPLOY_STATE_ANTENNA_VERIFY:
            /* Verify deployment */
            if (check_element_deployed(DEPLOY_ELEMENT_ANTENNA)) {
                antenna->deployed = true;
                antenna->deploy_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_SOLAR_PENDING, current_time_ms);
                LOG_INFO("DEPLOY", "Antenna deployment confirmed");
            } else if (time_in_state >= DEPLOYMENT_VERIFY_TIMEOUT_MS) {
                /* Retry after delay */
                if (time_in_state >= DEPLOYMENT_RETRY_DELAY_MS) {
                    enter_state(DEPLOY_STATE_ANTENNA_PENDING, current_time_ms);
                }
            }
            break;

        case DEPLOY_STATE_SOLAR_PENDING:
            /* Check if already deployed */
            if (check_element_deployed(DEPLOY_ELEMENT_SOLAR_PANEL)) {
                solar->deployed = true;
                solar->deploy_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_COMPLETE, current_time_ms);
                LOG_INFO("DEPLOY", "Solar panels already deployed");
            } else if (solar->attempt_count < DEPLOYMENT_MAX_RETRIES) {
                /* Start deployment */
                actuate_deployment(DEPLOY_ELEMENT_SOLAR_PANEL, true);
                solar->deploy_attempted = true;
                solar->attempt_count++;
                s_deploy_status.deploy_start_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_SOLAR_ACTIVE, current_time_ms);
                LOG_INFO("DEPLOY", "Solar deploy attempt %u",
                         (unsigned)solar->attempt_count);
            } else {
                /* Max retries exceeded - but antenna worked, partial success */
                fault_log_add(FAULT_TYPE_DEPLOYMENT, FAULT_SEVERITY_ERROR,
                             "Solar panel deployment failed", 0.0);
                enter_state(DEPLOY_STATE_FAILED, current_time_ms);
                LOG_ERROR("DEPLOY", "Solar deploy failed after %u attempts",
                          (unsigned)DEPLOYMENT_MAX_RETRIES);
            }
            break;

        case DEPLOY_STATE_SOLAR_ACTIVE:
            /* Monitor burn time */
            if (time_in_state >= SOLAR_DEPLOY_BURN_TIME_MS) {
                actuate_deployment(DEPLOY_ELEMENT_SOLAR_PANEL, false);
                enter_state(DEPLOY_STATE_SOLAR_VERIFY, current_time_ms);
            }
            break;

        case DEPLOY_STATE_SOLAR_VERIFY:
            /* Verify deployment */
            if (check_element_deployed(DEPLOY_ELEMENT_SOLAR_PANEL)) {
                solar->deployed = true;
                solar->deploy_time_ms = current_time_ms;
                enter_state(DEPLOY_STATE_COMPLETE, current_time_ms);
                LOG_INFO("DEPLOY", "All deployments complete");
            } else if (time_in_state >= DEPLOYMENT_VERIFY_TIMEOUT_MS) {
                /* Retry after delay */
                if (time_in_state >= DEPLOYMENT_RETRY_DELAY_MS) {
                    enter_state(DEPLOY_STATE_SOLAR_PENDING, current_time_ms);
                }
            }
            break;

        case DEPLOY_STATE_COMPLETE:
            /* Deployment complete, save state */
            (void)deployment_save_state();
            break;

        case DEPLOY_STATE_FAILED:
            /* Deployment failed, wait for ground command */
            break;

        case DEPLOY_STATE_COUNT:
            /* Fall through to default */
        default:
            /* Invalid state, reset to inhibited */
            enter_state(DEPLOY_STATE_INHIBITED, current_time_ms);
            fault_log_add(FAULT_TYPE_SW_INTERNAL, FAULT_SEVERITY_ERROR,
                         "Invalid deployment state", 0.0);
            break;
    }

    /* Update switch states for telemetry */
    antenna->switch_state = check_element_deployed(DEPLOY_ELEMENT_ANTENNA);
    solar->switch_state = check_element_deployed(DEPLOY_ELEMENT_SOLAR_PANEL);
}

bool deployment_is_separated(void) {
    return s_deploy_status.separation_detected;
}

DeploymentState_t deployment_get_state(void) {
    return s_deploy_status.state;
}

bool deployment_get_element_status(DeploymentElement_t element,
                                   DeploymentElementStatus_t *status) {
    if (element >= DEPLOY_ELEMENT_COUNT || status == NULL) {
        return false;
    }

    *status = s_deploy_status.elements[element];
    return true;
}

bool deployment_get_telemetry(DeploymentTelemetry_t *telemetry) {
    if (telemetry == NULL) {
        return false;
    }

    telemetry->state = (uint8_t)s_deploy_status.state;
    telemetry->antenna_deployed = s_deploy_status.elements[DEPLOY_ELEMENT_ANTENNA].deployed ? 1U : 0U;
    telemetry->antenna_attempts = s_deploy_status.elements[DEPLOY_ELEMENT_ANTENNA].attempt_count;
    telemetry->solar_deployed = s_deploy_status.elements[DEPLOY_ELEMENT_SOLAR_PANEL].deployed ? 1U : 0U;
    telemetry->solar_attempts = s_deploy_status.elements[DEPLOY_ELEMENT_SOLAR_PANEL].attempt_count;

    if (s_deploy_status.separation_detected) {
        uint64_t current = hal_timer_get_ms();
        telemetry->time_since_sep_s = (uint32_t)((current - s_deploy_status.separation_time_ms) / 1000U);
    } else {
        telemetry->time_since_sep_s = 0;
    }

    telemetry->inhibit_active = !s_deploy_status.deployment_enabled ? 1U : 0U;

    return true;
}

bool deployment_set_enabled(bool enable) {
    s_deploy_status.deployment_enabled = enable;
    LOG_INFO("DEPLOY", "Deployment %s by command",
             enable ? "enabled" : "inhibited");
    return true;
}

bool deployment_force_retry(DeploymentElement_t element) {
    if (element >= DEPLOY_ELEMENT_COUNT) {
        return false;
    }

    /* Reset attempt counter and force re-entry to pending state */
    s_deploy_status.elements[element].attempt_count = 0;
    s_deploy_status.elements[element].deployed = false;

    uint32_t current_time = (uint32_t)hal_timer_get_ms();

    if (element == DEPLOY_ELEMENT_ANTENNA) {
        enter_state(DEPLOY_STATE_ANTENNA_PENDING, current_time);
    } else {
        enter_state(DEPLOY_STATE_SOLAR_PENDING, current_time);
    }

    LOG_INFO("DEPLOY", "Forced retry for element %u", (unsigned)element);
    return true;
}

const char *deployment_state_name(DeploymentState_t state) {
    if (state < DEPLOY_STATE_COUNT) {
        return s_state_names[state];
    }
    return "UNKNOWN";
}

bool deployment_all_complete(void) {
    return s_deploy_status.elements[DEPLOY_ELEMENT_ANTENNA].deployed &&
           s_deploy_status.elements[DEPLOY_ELEMENT_SOLAR_PANEL].deployed;
}

bool deployment_save_state(void) {
    s_deploy_status.crc = calculate_crc(&s_deploy_status);

    /* In real implementation, write to FRAM/Flash */
    /* For simulation, state is kept in RAM */
    LOG_INFO("DEPLOY", "State saved");
    return true;
}

bool deployment_load_state(void) {
    /* In real implementation, read from FRAM/Flash and verify CRC */
    /* For simulation, return false to indicate no saved state */
    return false;
}

/*===========================================================================*/
/* Private Functions                                                          */
/*===========================================================================*/

static void enter_state(DeploymentState_t new_state, uint32_t current_time_ms) {
    DeploymentState_t old_state = s_deploy_status.state;
    s_deploy_status.state = new_state;
    s_deploy_status.state_entry_time_ms = current_time_ms;

    LOG_DEBUG("DEPLOY", "State: %s -> %s",
              deployment_state_name(old_state),
              deployment_state_name(new_state));
}

static bool check_deployment_switch(void) {
    /* Deployment switch is normally closed, opens on separation */
    /* In simulation, return true after first call to simulate separation */
#ifdef SIMULATION_BUILD
    static uint32_t sim_counter = 0;
    sim_counter++;
    /* Simulate separation after 5 seconds */
    return (sim_counter > 50);
#else
    return !hal_gpio_get(DEPLOY_SWITCH_PIN);
#endif
}

static bool check_element_deployed(DeploymentElement_t element) {
#ifdef SIMULATION_BUILD
    /* In simulation, assume deployed after actuation attempt */
    return s_deploy_status.elements[element].deploy_attempted;
#else
    (void)element;
    return hal_gpio_get(GPIO_PIN_DEPLOY_SENSE);
#endif
}

static void actuate_deployment(DeploymentElement_t element, bool activate) {
    (void)hal_gpio_set(GPIO_PIN_DEPLOY_EN, activate);

    LOG_DEBUG("DEPLOY", "Burn wire %s: %s",
              (element == DEPLOY_ELEMENT_ANTENNA) ? "ANTENNA" : "SOLAR",
              activate ? "ON" : "OFF");
}

static uint32_t calculate_crc(const DeploymentStatus_t *status) {
    /* Calculate CRC excluding the CRC field itself */
    size_t crc_offset = offsetof(DeploymentStatus_t, crc);
    return smart_qso_crc32(status, crc_offset);
}
