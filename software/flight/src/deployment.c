/**
 * @file deployment.c
 * @brief SMART-QSO Deployment Control Module Implementation
 *
 * Implements deployment sequencing for antenna and solar panel deployment
 * following CubeSat Design Specification requirements and NASA Ames
 * SmallSat heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat).
 *
 * Reset and persistence rules are described in deployment.h.
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 *
 * @requirement SYS-221 Deployables wait 30 minutes after separation; switch toggle resets the timer
 */

#include "deployment.h"
#include "separation_switch.h"
#include "nvm_record.h"
#include "fault_mgmt.h"
#include "flight_log.h"
#include "smart_qso.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_reset.h"

#include <string.h>
#include <stdint.h>

/*===========================================================================*/
/* Private Constants                                                          */
/*===========================================================================*/

/** Persisted record identifier ("DEPL") */
#define RECORD_MAGIC            0x4445504CU

/** Record flags */
#define FLAG_ANTENNA_DEPLOYED   0x01U
#define FLAG_SOLAR_DEPLOYED     0x02U
#define FLAG_ENABLED            0x04U
#define FLAG_SEPARATED          0x08U

/** Persisted record: state | attempts (2) | flags | wait seconds */
typedef struct {
    uint8_t state;
    uint8_t attempts[DEPLOY_ELEMENT_COUNT];
    uint8_t flags;
    uint32_t wait_elapsed_s;
} DeployRecord_t;

/** Per-element pins and burn times. TBR: sense polarity (high = deployed)
 *  and pin assignment, against the schematic */
static const HalGpioPin_t BURN_PIN[DEPLOY_ELEMENT_COUNT] = {
    GPIO_PIN_ANTENNA_BURN, GPIO_PIN_SOLAR_BURN
};
static const HalGpioPin_t SENSE_PIN[DEPLOY_ELEMENT_COUNT] = {
    GPIO_PIN_ANTENNA_SENSE, GPIO_PIN_SOLAR_SENSE
};
static const uint32_t BURN_TIME_MS[DEPLOY_ELEMENT_COUNT] = {
    ANTENNA_DEPLOY_BURN_TIME_MS, SOLAR_DEPLOY_BURN_TIME_MS
};

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

static DeploymentStatus_t s_deploy;
static uint32_t s_unpersisted_s = 0U;
static uint32_t s_wait_remainder_ms = 0U;
static uint32_t s_sep_remainder_ms = 0U;
static bool s_persist_fault_logged = false;
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
    "FAILED",
    "ARMED"
};

/*===========================================================================*/
/* Private Functions                                                          */
/*===========================================================================*/

/** Master arm off first, then both element lines */
static void burn_off(void)
{
    (void)hal_gpio_set(GPIO_PIN_DEPLOY_EN, false);
    (void)hal_gpio_set(GPIO_PIN_ANTENNA_BURN, false);
    (void)hal_gpio_set(GPIO_PIN_SOLAR_BURN, false);
}

/** Element line first, then the master arm: both are needed to burn */
static void burn_on(DeploymentElement_t element)
{
    (void)hal_gpio_set(BURN_PIN[element], true);
    (void)hal_gpio_set(GPIO_PIN_DEPLOY_EN, true);
}

static bool element_sensed(DeploymentElement_t element)
{
    return hal_gpio_get(SENSE_PIN[element]);
}

static bool save(void)
{
    DeployRecord_t rec;

    (void)memset(&rec, 0, sizeof(rec));
    rec.state = (uint8_t)s_deploy.state;
    for (uint32_t e = 0U; e < (uint32_t)DEPLOY_ELEMENT_COUNT; e++) {
        rec.attempts[e] = s_deploy.elements[e].attempt_count;
    }
    rec.flags = (uint8_t)((s_deploy.elements[DEPLOY_ELEMENT_ANTENNA].deployed ? FLAG_ANTENNA_DEPLOYED : 0U) |
                          (s_deploy.elements[DEPLOY_ELEMENT_SOLAR_PANEL].deployed ? FLAG_SOLAR_DEPLOYED : 0U) |
                          (s_deploy.deployment_enabled ? FLAG_ENABLED : 0U) |
                          (s_deploy.separation_detected ? FLAG_SEPARATED : 0U));
    rec.wait_elapsed_s = s_deploy.wait_elapsed_s;

    if (nvm_record_save(FLASH_REGION_DEPLOYMENT, RECORD_MAGIC, &rec, sizeof(rec)) != SMART_QSO_OK) {
        return false;
    }
    s_unpersisted_s = 0U;
    s_persist_fault_logged = false;
    return true;
}

static void enter_state(DeploymentState_t new_state)
{
    LOG_DEBUG("DEPLOY", "State: %s -> %s",
              deployment_state_name(s_deploy.state), deployment_state_name(new_state));
    s_deploy.state = new_state;
    s_deploy.state_elapsed_ms = 0U;
}

/** Back to waiting for separation with the 30-minute count at zero */
static void restart_wait(void)
{
    s_deploy.wait_elapsed_s = 0U;
    s_deploy.separation_detected = false;
    s_wait_remainder_ms = 0U;
    enter_state(DEPLOY_STATE_INHIBITED);
}

/** Start from nothing: no attempts, nothing deployed, enabled */
static void fresh_start(void)
{
    (void)memset(&s_deploy, 0, sizeof(s_deploy));
    s_deploy.deployment_enabled = true;
    restart_wait();
}

static DeploymentState_t next_after(DeploymentElement_t element)
{
    return (element == DEPLOY_ELEMENT_ANTENNA) ? DEPLOY_STATE_SOLAR_PENDING : DEPLOY_STATE_COMPLETE;
}

static void mark_deployed(DeploymentElement_t element)
{
    s_deploy.elements[element].deployed = true;
    s_deploy.elements[element].deploy_time_s = s_deploy.since_separation_s;
    enter_state(next_after(element));
    (void)save();
    LOG_INFO("DEPLOY", "Element %u deployed", (unsigned)element);
}

static void process_pending(DeploymentElement_t element, DeploymentState_t active_state)
{
    DeploymentElementStatus_t *el = &s_deploy.elements[element];

    /* Idempotent: never fire an element that is already out */
    if (el->deployed || element_sensed(element)) {
        mark_deployed(element);
        return;
    }
    if (!s_deploy.deployment_enabled) {
        return;
    }
    if (el->attempt_count >= DEPLOYMENT_MAX_RETRIES) {
        (void)fault_log_add(FAULT_TYPE_DEPLOYMENT, FAULT_SEVERITY_ERROR,
                            (element == DEPLOY_ELEMENT_ANTENNA) ? "Antenna deployment failed"
                                                               : "Solar panel deployment failed", 0.0);
        enter_state(DEPLOY_STATE_FAILED);
        (void)save();
        return;
    }

    /* Persist the attempt (and the burning state) before driving the pin, so
     * a reset can never repeat a burn; if it cannot be persisted, do not burn */
    DeploymentState_t previous = s_deploy.state;
    bool previously_attempted = el->deploy_attempted;
    el->attempt_count++;
    el->deploy_attempted = true;
    enter_state(active_state);
    if (!save()) {
        el->attempt_count--;
        el->deploy_attempted = previously_attempted;
        s_deploy.state = previous;
        if (!s_persist_fault_logged) {
            s_persist_fault_logged = true;
            (void)fault_log_add(FAULT_TYPE_DEPLOYMENT, FAULT_SEVERITY_ERROR,
                                "Deployment attempt not persisted: burn withheld", 0.0);
        }
        return;
    }

    burn_on(element);
    LOG_INFO("DEPLOY", "Element %u burn attempt %u", (unsigned)element, (unsigned)el->attempt_count);
}

static void process_active(DeploymentElement_t element, DeploymentState_t verify_state)
{
    if (s_deploy.state_elapsed_ms >= BURN_TIME_MS[element]) {
        burn_off();
        enter_state(verify_state);
        (void)save();
    }
}

static void process_verify(DeploymentElement_t element, DeploymentState_t pending_state)
{
    if (element_sensed(element)) {
        mark_deployed(element);
    } else if (s_deploy.state_elapsed_ms >= DEPLOYMENT_RETRY_DELAY_MS) {
        enter_state(pending_state);
        (void)save();
    } else {
        /* Keep waiting for the sense line */
    }
}

static void process_waiting(uint32_t elapsed_ms)
{
    uint32_t total_ms = s_wait_remainder_ms + elapsed_ms;
    uint32_t seconds = total_ms / 1000U;
    s_wait_remainder_ms = total_ms % 1000U;
    s_deploy.wait_elapsed_s += seconds;
    s_unpersisted_s += seconds;

    if ((s_deploy.wait_elapsed_s >= DEPLOYMENT_WAIT_TIME_S) && s_deploy.deployment_enabled) {
        enter_state(DEPLOY_STATE_ANTENNA_PENDING);
        (void)save();
        LOG_INFO("DEPLOY", "Wait complete, starting antenna deploy");
    } else if (s_unpersisted_s >= DEPLOYMENT_PERSIST_INTERVAL_S) {
        (void)save();
    } else {
        /* Persist at the next interval */
    }
}

/*===========================================================================*/
/* Public Functions                                                           */
/*===========================================================================*/

bool deployment_init(void)
{
    DeployRecord_t rec;
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;
    HalResetCause_t cause = hal_reset_cause();
    bool power_lost = (cause != HAL_RESET_WATCHDOG) && (cause != HAL_RESET_SOFTWARE);

    /* Burn lines low before anything else */
    (void)hal_gpio_config(GPIO_PIN_DEPLOY_EN, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN);
    (void)hal_gpio_config(GPIO_PIN_ANTENNA_BURN, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN);
    (void)hal_gpio_config(GPIO_PIN_SOLAR_BURN, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN);
    burn_off();
    (void)hal_gpio_config(GPIO_PIN_ANTENNA_SENSE, GPIO_DIR_INPUT, GPIO_PULL_DOWN);
    (void)hal_gpio_config(GPIO_PIN_SOLAR_SENSE, GPIO_DIR_INPUT, GPIO_PULL_DOWN);
    separation_switch_init();

    fresh_start();
    s_unpersisted_s = 0U;
    s_sep_remainder_ms = 0U;
    s_persist_fault_logged = false;

    SmartQsoResult_t result = nvm_record_load(FLASH_REGION_DEPLOYMENT, RECORD_MAGIC,
                                              &rec, sizeof(rec), &status);
    if ((result == SMART_QSO_OK) && (status == NVM_RECORD_VALID) &&
        (rec.state < (uint8_t)DEPLOY_STATE_COUNT)) {
        DeploymentState_t state = (DeploymentState_t)rec.state;

        if (state != DEPLOY_STATE_ARMED) {
            /* ARMED: the sequence starts fresh at this boot */
            for (uint32_t e = 0U; e < (uint32_t)DEPLOY_ELEMENT_COUNT; e++) {
                s_deploy.elements[e].attempt_count = rec.attempts[e];
                s_deploy.elements[e].deploy_attempted = (rec.attempts[e] > 0U);
            }
            s_deploy.elements[DEPLOY_ELEMENT_ANTENNA].deployed = ((rec.flags & FLAG_ANTENNA_DEPLOYED) != 0U);
            s_deploy.elements[DEPLOY_ELEMENT_SOLAR_PANEL].deployed = ((rec.flags & FLAG_SOLAR_DEPLOYED) != 0U);
            s_deploy.deployment_enabled = ((rec.flags & FLAG_ENABLED) != 0U);

            if ((state == DEPLOY_STATE_COMPLETE) || (state == DEPLOY_STATE_FAILED)) {
                s_deploy.state = state;
                s_deploy.separation_detected = ((rec.flags & FLAG_SEPARATED) != 0U);
            } else if (power_lost) {
                /* Power removed (a switch toggle does this): the wait restarts */
            } else {
                s_deploy.separation_detected = ((rec.flags & FLAG_SEPARATED) != 0U);
                s_deploy.wait_elapsed_s = rec.wait_elapsed_s;
                /* A reset during a burn spends the attempt: verify, never re-fire */
                if (state == DEPLOY_STATE_ANTENNA_ACTIVE) {
                    state = DEPLOY_STATE_ANTENNA_VERIFY;
                } else if (state == DEPLOY_STATE_SOLAR_ACTIVE) {
                    state = DEPLOY_STATE_SOLAR_VERIFY;
                } else {
                    /* Resume where it was */
                }
                s_deploy.state = state;
            }
        }
    } else if ((result != SMART_QSO_OK) || (status != NVM_RECORD_BLANK)) {
        (void)fault_log_add(FAULT_TYPE_DEPLOYMENT, FAULT_SEVERITY_ERROR,
                            "Deployment record unreadable: starting over", 0.0);
    } else {
        /* Blank: first boot */
    }

    (void)save();
    s_initialized = true;
    LOG_INFO("DEPLOY", "Deployment initialized in %s", deployment_state_name(s_deploy.state));
    return true;
}

/** Advance the timers and read the deploy-sense switches */
static void update_timers_and_sensors(uint32_t elapsed_ms)
{
    s_deploy.state_elapsed_ms = ((UINT32_MAX - s_deploy.state_elapsed_ms) < elapsed_ms)
                                    ? UINT32_MAX : (s_deploy.state_elapsed_ms + elapsed_ms);
    if (s_deploy.separation_detected) {
        uint32_t total_ms = s_sep_remainder_ms + elapsed_ms;
        s_deploy.since_separation_s += total_ms / 1000U;
        s_sep_remainder_ms = total_ms % 1000U;
    }
    for (uint32_t e = 0U; e < (uint32_t)DEPLOY_ELEMENT_COUNT; e++) {
        s_deploy.elements[e].switch_state = element_sensed((DeploymentElement_t)e);
    }
}

/** One step of the burn sequence for the current state */
static void step_sequence(uint32_t elapsed_ms)
{
    switch (s_deploy.state) {
        case DEPLOY_STATE_INHIBITED:
            s_deploy.separation_detected = true;
            s_deploy.since_separation_s = 0U;
            enter_state(DEPLOY_STATE_WAITING);
            (void)save();
            LOG_INFO("DEPLOY", "Separation detected");
            break;
        case DEPLOY_STATE_WAITING:
            process_waiting(elapsed_ms);
            break;
        case DEPLOY_STATE_ANTENNA_PENDING:
            process_pending(DEPLOY_ELEMENT_ANTENNA, DEPLOY_STATE_ANTENNA_ACTIVE);
            break;
        case DEPLOY_STATE_ANTENNA_ACTIVE:
            process_active(DEPLOY_ELEMENT_ANTENNA, DEPLOY_STATE_ANTENNA_VERIFY);
            break;
        case DEPLOY_STATE_ANTENNA_VERIFY:
            process_verify(DEPLOY_ELEMENT_ANTENNA, DEPLOY_STATE_ANTENNA_PENDING);
            break;
        case DEPLOY_STATE_SOLAR_PENDING:
            process_pending(DEPLOY_ELEMENT_SOLAR_PANEL, DEPLOY_STATE_SOLAR_ACTIVE);
            break;
        case DEPLOY_STATE_SOLAR_ACTIVE:
            process_active(DEPLOY_ELEMENT_SOLAR_PANEL, DEPLOY_STATE_SOLAR_VERIFY);
            break;
        case DEPLOY_STATE_SOLAR_VERIFY:
            process_verify(DEPLOY_ELEMENT_SOLAR_PANEL, DEPLOY_STATE_SOLAR_PENDING);
            break;
        case DEPLOY_STATE_COMPLETE:
        case DEPLOY_STATE_FAILED:
        case DEPLOY_STATE_ARMED:
            break;  /* terminal or waiting for a boot: handled by the caller */
        case DEPLOY_STATE_COUNT:
        default:
            /* Invalid state: stop any burn and start over */
            burn_off();
            restart_wait();
            (void)save();
            (void)fault_log_add(FAULT_TYPE_SW_INTERNAL, FAULT_SEVERITY_ERROR,
                                "Invalid deployment state", 0.0);
            break;
    }
}

void deployment_tick(uint32_t elapsed_ms)
{
    if (!s_initialized) {
        return;
    }
    update_timers_and_sensors(elapsed_ms);

    /* Terminal states ignore the switch: re-burning is not harmless.
     * ARMED waits for the next boot. */
    if ((s_deploy.state == DEPLOY_STATE_COMPLETE) || (s_deploy.state == DEPLOY_STATE_FAILED) ||
        (s_deploy.state == DEPLOY_STATE_ARMED)) {
        return;
    }

    /* CDS 2.3.2.4: a stowed switch restarts the wait (and stops any burn) */
    if (separation_switch_stowed()) {
        if ((s_deploy.state != DEPLOY_STATE_INHIBITED) || (s_deploy.wait_elapsed_s != 0U)) {
            burn_off();
            restart_wait();
            (void)save();
        }
        return;
    }

    step_sequence(elapsed_ms);
}

SmartQsoResult_t deployment_arm(void)
{
    burn_off();
    fresh_start();
    s_deploy.state = DEPLOY_STATE_ARMED;
    return save() ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

bool deployment_is_separated(void)
{
    return s_deploy.separation_detected;
}

DeploymentState_t deployment_get_state(void)
{
    return s_deploy.state;
}

uint32_t deployment_wait_elapsed_s(void)
{
    return s_deploy.wait_elapsed_s;
}

bool deployment_get_element_status(DeploymentElement_t element,
                                   DeploymentElementStatus_t *status)
{
    if ((element >= DEPLOY_ELEMENT_COUNT) || (status == NULL)) {
        return false;
    }

    *status = s_deploy.elements[element];
    return true;
}

bool deployment_get_telemetry(DeploymentTelemetry_t *telemetry)
{
    if (telemetry == NULL) {
        return false;
    }

    telemetry->state = (uint8_t)s_deploy.state;
    telemetry->antenna_deployed = s_deploy.elements[DEPLOY_ELEMENT_ANTENNA].deployed ? 1U : 0U;
    telemetry->antenna_attempts = s_deploy.elements[DEPLOY_ELEMENT_ANTENNA].attempt_count;
    telemetry->solar_deployed = s_deploy.elements[DEPLOY_ELEMENT_SOLAR_PANEL].deployed ? 1U : 0U;
    telemetry->solar_attempts = s_deploy.elements[DEPLOY_ELEMENT_SOLAR_PANEL].attempt_count;
    telemetry->time_since_sep_s = s_deploy.separation_detected ? s_deploy.since_separation_s : 0U;
    telemetry->inhibit_active = !s_deploy.deployment_enabled ? 1U : 0U;

    return true;
}

bool deployment_set_enabled(bool enable)
{
    s_deploy.deployment_enabled = enable;
    LOG_INFO("DEPLOY", "Deployment %s by command", enable ? "enabled" : "inhibited");
    return save();
}

bool deployment_force_retry(DeploymentElement_t element)
{
    if (element >= DEPLOY_ELEMENT_COUNT) {
        return false;
    }

    /* Never before the 30-minute wait (CDS 2.4.4), and never mid-burn */
    switch (s_deploy.state) {
        case DEPLOY_STATE_INHIBITED:
        case DEPLOY_STATE_WAITING:
        case DEPLOY_STATE_ARMED:
        case DEPLOY_STATE_ANTENNA_ACTIVE:
        case DEPLOY_STATE_SOLAR_ACTIVE:
        case DEPLOY_STATE_COUNT:
            return false;
        case DEPLOY_STATE_ANTENNA_PENDING:
        case DEPLOY_STATE_ANTENNA_VERIFY:
        case DEPLOY_STATE_SOLAR_PENDING:
        case DEPLOY_STATE_SOLAR_VERIFY:
        case DEPLOY_STATE_COMPLETE:
        case DEPLOY_STATE_FAILED:
        default:
            break;
    }

    s_deploy.elements[element].attempt_count = 0U;
    s_deploy.elements[element].deployed = false;
    enter_state((element == DEPLOY_ELEMENT_ANTENNA) ? DEPLOY_STATE_ANTENNA_PENDING
                                                    : DEPLOY_STATE_SOLAR_PENDING);
    (void)save();

    LOG_INFO("DEPLOY", "Forced retry for element %u", (unsigned)element);
    return true;
}

const char *deployment_state_name(DeploymentState_t state)
{
    if (state < DEPLOY_STATE_COUNT) {
        return s_state_names[state];
    }
    return "UNKNOWN";
}

bool deployment_all_complete(void)
{
    return s_deploy.elements[DEPLOY_ELEMENT_ANTENNA].deployed &&
           s_deploy.elements[DEPLOY_ELEMENT_SOLAR_PANEL].deployed;
}

bool deployment_save_state(void)
{
    return save();
}
