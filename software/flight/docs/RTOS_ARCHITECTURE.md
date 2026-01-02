# RTOS Architecture
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-011
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, FreeRTOS Documentation, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the real-time operating system architecture for SMART-QSO flight software, including task structure, synchronization, and resource management.

## 2. Scope

This document covers:
- FreeRTOS configuration
- Task architecture
- Inter-task communication
- Timing and scheduling
- Resource management

---

## 3. RTOS Selection

### 3.1 FreeRTOS Justification

| Criterion | FreeRTOS | Rationale |
|-----------|----------|-----------|
| Footprint | ~10 KB | Fits in constrained memory |
| Real-time | Preemptive | Deterministic response |
| Certification | SAFERTOS available | Safety-critical variant |
| Heritage | Wide CubeSat use | GeneSat, PharmaSat heritage |
| License | MIT | No export restrictions |

### 3.2 Version

| Component | Version |
|-----------|---------|
| FreeRTOS Kernel | 10.5.1 |
| CMSIS-RTOS2 | 2.1.3 |
| Target | STM32L4R5 (Cortex-M4) |

---

## 4. Task Architecture

### 4.1 Task Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        TASK HIERARCHY                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Priority 4 (Highest)                                        │
│  ┌──────────────┐                                           │
│  │  Watchdog    │  Period: 10s   Stack: 256B                │
│  │  Task        │  Deadline: 10s                            │
│  └──────────────┘                                           │
│                                                              │
│  Priority 3 (High)                                           │
│  ┌──────────────┐  ┌──────────────┐                         │
│  │   Sensor     │  │    ADCS      │                         │
│  │   Task       │  │   Control    │                         │
│  │ Period: 100ms│  │ Period: 1s   │                         │
│  └──────────────┘  └──────────────┘                         │
│                                                              │
│  Priority 2 (Normal)                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │    Main      │  │   Beacon     │  │   Command    │       │
│  │    Loop      │  │  Generator   │  │   Handler    │       │
│  │ Period: 100ms│  │ Period: var  │  │ Event-driven │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│                                                              │
│  Priority 1 (Low)                                            │
│  ┌──────────────┐  ┌──────────────┐                         │
│  │  Telemetry   │  │ Housekeeping │                         │
│  │  Collector   │  │    Task      │                         │
│  │ Period: 5s   │  │ Period: 10s  │                         │
│  └──────────────┘  └──────────────┘                         │
│                                                              │
│  Priority 0 (Idle)                                           │
│  ┌──────────────┐                                           │
│  │    Idle      │  Background scrubbing, power save         │
│  │    Task      │                                           │
│  └──────────────┘                                           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Task Table

| Task | Priority | Period | Stack | Purpose |
|------|----------|--------|-------|---------|
| Watchdog | 4 | 10 s | 256 B | Kick hardware WDT |
| Sensor | 3 | 100 ms | 1 KB | Sample all sensors |
| ADCS Control | 3 | 1000 ms | 2 KB | Attitude control |
| Main Loop | 2 | 100 ms | 4 KB | System management |
| Beacon | 2 | Variable | 2 KB | Generate beacons |
| Command | 2 | Event | 2 KB | Process uplinks |
| Telemetry | 1 | 5000 ms | 1 KB | Collect telemetry |
| Housekeeping | 1 | 10000 ms | 512 B | Maintenance |
| Idle | 0 | - | 128 B | Background work |

### 4.3 Task Creation

```c
/* Task handle declarations */
static TaskHandle_t s_watchdog_task;
static TaskHandle_t s_sensor_task;
static TaskHandle_t s_adcs_task;
static TaskHandle_t s_main_task;
static TaskHandle_t s_beacon_task;
static TaskHandle_t s_command_task;
static TaskHandle_t s_telemetry_task;
static TaskHandle_t s_housekeeping_task;

/* Task creation in system_init() */
void create_tasks(void)
{
    xTaskCreate(watchdog_task_fn, "WDT",
                configMINIMAL_STACK_SIZE, NULL,
                4, &s_watchdog_task);

    xTaskCreate(sensor_task_fn, "SENS",
                256, NULL,
                3, &s_sensor_task);

    xTaskCreate(adcs_task_fn, "ADCS",
                512, NULL,
                3, &s_adcs_task);

    xTaskCreate(main_task_fn, "MAIN",
                1024, NULL,
                2, &s_main_task);

    /* ... additional tasks ... */
}
```

---

## 5. Inter-Task Communication

### 5.1 Communication Mechanisms

| Mechanism | Use Case | Tasks |
|-----------|----------|-------|
| Queue | Sensor data | Sensor → Main |
| Queue | Commands | Command → Main |
| Mutex | Telemetry buffer | All → Telemetry |
| Semaphore | Beacon trigger | Main → Beacon |
| Event Groups | Mode changes | Main → All |
| Direct Notify | Watchdog kick | Watchdog → All |

### 5.2 Queue Architecture

```
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   Sensor    │────▶│  Sensor Queue   │────▶│    Main     │
│    Task     │     │  Depth: 4       │     │    Task     │
└─────────────┘     │  Item: 64 bytes │     └─────────────┘
                    └─────────────────┘

┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   Command   │────▶│  Command Queue  │────▶│    Main     │
│    Task     │     │  Depth: 8       │     │    Task     │
└─────────────┘     │  Item: 128 bytes│     └─────────────┘
                    └─────────────────┘

┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│    Main     │────▶│  Beacon Queue   │────▶│   Beacon    │
│    Task     │     │  Depth: 2       │     │    Task     │
└─────────────┘     │  Item: 256 bytes│     └─────────────┘
                    └─────────────────┘
```

### 5.3 Queue Definitions

```c
/* Queue handles */
static QueueHandle_t s_sensor_queue;
static QueueHandle_t s_command_queue;
static QueueHandle_t s_beacon_queue;

/* Queue item structures */
typedef struct {
    uint32_t timestamp;
    Vector3_t magnetometer;
    float sun_sensors[6];
    float temperature;
} SensorQueueItem_t;

typedef struct {
    uint8_t command_id;
    uint8_t payload[127];
} CommandQueueItem_t;

/* Queue creation */
void create_queues(void)
{
    s_sensor_queue = xQueueCreate(4, sizeof(SensorQueueItem_t));
    s_command_queue = xQueueCreate(8, sizeof(CommandQueueItem_t));
    s_beacon_queue = xQueueCreate(2, sizeof(BeaconData_t));
}
```

### 5.4 Mutex Usage

```c
/* Telemetry buffer mutex */
static SemaphoreHandle_t s_telemetry_mutex;

/* Protected access */
void telemetry_update(const TelemetryPacket_t *packet)
{
    if (xSemaphoreTake(s_telemetry_mutex, pdMS_TO_TICKS(100)) == pdTRUE) {
        memcpy(&g_telemetry_buffer, packet, sizeof(TelemetryPacket_t));
        xSemaphoreGive(s_telemetry_mutex);
    } else {
        fault_mgmt_report(FAULT_MUTEX_TIMEOUT);
    }
}
```

---

## 6. Timing and Scheduling

### 6.1 FreeRTOS Configuration

```c
/* FreeRTOSConfig.h key settings */
#define configUSE_PREEMPTION                    1
#define configUSE_PORT_OPTIMISED_TASK_SELECTION 1
#define configUSE_TICKLESS_IDLE                 0
#define configCPU_CLOCK_HZ                      80000000
#define configTICK_RATE_HZ                      1000
#define configMAX_PRIORITIES                    5
#define configMINIMAL_STACK_SIZE                128
#define configTOTAL_HEAP_SIZE                   32768
#define configMAX_TASK_NAME_LEN                 8
#define configUSE_MUTEXES                       1
#define configUSE_COUNTING_SEMAPHORES           1
#define configUSE_QUEUE_SETS                    0
#define configUSE_TASK_NOTIFICATIONS            1
#define configTASK_NOTIFICATION_ARRAY_ENTRIES   1
#define configUSE_TRACE_FACILITY                1
#define configUSE_STATS_FORMATTING_FUNCTIONS    1
```

### 6.2 Periodic Task Pattern

```c
/* Standard periodic task implementation */
void sensor_task_fn(void *pvParameters)
{
    (void)pvParameters;
    TickType_t xLastWakeTime = xTaskGetTickCount();
    const TickType_t xPeriod = pdMS_TO_TICKS(100);

    while (1) {
        /* Wait for next period */
        vTaskDelayUntil(&xLastWakeTime, xPeriod);

        /* Task work */
        SensorQueueItem_t data;
        if (sensors_read_all(&data) == RESULT_OK) {
            xQueueSend(s_sensor_queue, &data, 0);
        }

        /* Notify watchdog */
        xTaskNotify(s_watchdog_task, TASK_SENSOR_ALIVE, eSetBits);
    }
}
```

### 6.3 Event-Driven Task Pattern

```c
/* Event-driven task (command handler) */
void command_task_fn(void *pvParameters)
{
    (void)pvParameters;
    CommandQueueItem_t cmd;

    while (1) {
        /* Block on queue indefinitely */
        if (xQueueReceive(s_command_queue, &cmd, portMAX_DELAY) == pdTRUE) {
            process_command(&cmd);
            xTaskNotify(s_watchdog_task, TASK_CMD_ALIVE, eSetBits);
        }
    }
}
```

---

## 7. Memory Management

### 7.1 Heap Configuration

| Setting | Value | Rationale |
|---------|-------|-----------|
| Heap scheme | heap_4.c | Coalescing, low fragmentation |
| Total heap | 32 KB | Sufficient for all objects |
| Stack overflow | Hook enabled | Detection and logging |

### 7.2 Static Allocation

```c
/* Prefer static allocation for critical tasks */
static StaticTask_t s_watchdog_tcb;
static StackType_t s_watchdog_stack[configMINIMAL_STACK_SIZE];

TaskHandle_t create_watchdog_task(void)
{
    return xTaskCreateStatic(
        watchdog_task_fn,
        "WDT",
        configMINIMAL_STACK_SIZE,
        NULL,
        4,
        s_watchdog_stack,
        &s_watchdog_tcb
    );
}
```

### 7.3 Stack Monitoring

```c
/* Stack high water mark monitoring */
void check_stack_usage(void)
{
    UBaseType_t high_water;

    high_water = uxTaskGetStackHighWaterMark(s_main_task);
    if (high_water < STACK_WARNING_THRESHOLD) {
        log_warning("Main task stack low: %u words", high_water);
    }
}
```

---

## 8. Watchdog Integration

### 8.1 Task Watchdog Pattern

```c
/* Watchdog notification bits */
#define TASK_MAIN_ALIVE      (1 << 0)
#define TASK_SENSOR_ALIVE    (1 << 1)
#define TASK_ADCS_ALIVE      (1 << 2)
#define TASK_BEACON_ALIVE    (1 << 3)
#define TASK_ALL_ALIVE       0x0F

void watchdog_task_fn(void *pvParameters)
{
    (void)pvParameters;
    uint32_t notification;
    const TickType_t xPeriod = pdMS_TO_TICKS(10000);

    while (1) {
        /* Wait for all tasks to check in */
        if (xTaskNotifyWait(0, TASK_ALL_ALIVE, &notification, xPeriod) == pdTRUE) {
            if ((notification & TASK_ALL_ALIVE) == TASK_ALL_ALIVE) {
                hal_watchdog_kick();
            } else {
                /* Log which task(s) didn't respond */
                log_fault(FAULT_TASK_TIMEOUT, ~notification & TASK_ALL_ALIVE);
            }
        } else {
            /* Timeout - no notification received */
            log_fault(FAULT_WATCHDOG_TIMEOUT, 0);
            /* Allow hardware watchdog to reset */
        }
    }
}
```

---

## 9. Power Management

### 9.1 Idle Task Hook

```c
/* Low power in idle */
void vApplicationIdleHook(void)
{
    /* Enter sleep mode */
    __WFI();  /* Wait for interrupt */
}
```

### 9.2 Tickless Idle (Optional)

```c
/* For extended low power (not used in baseline) */
#if configUSE_TICKLESS_IDLE
void vPortSuppressTicksAndSleep(TickType_t xExpectedIdleTime)
{
    /* Configure RTC for wakeup */
    enter_stop_mode(xExpectedIdleTime);
}
#endif
```

---

## 10. Error Handling

### 10.1 Stack Overflow Hook

```c
void vApplicationStackOverflowHook(TaskHandle_t xTask, char *pcTaskName)
{
    /* Log and attempt recovery */
    fault_mgmt_report(FAULT_STACK_OVERFLOW);
    log_critical("Stack overflow: %s", pcTaskName);

    /* Reset via watchdog */
    while (1) {
        /* Wait for watchdog reset */
    }
}
```

### 10.2 Malloc Failed Hook

```c
void vApplicationMallocFailedHook(void)
{
    fault_mgmt_report(FAULT_MALLOC_FAILED);
    log_critical("Malloc failed");

    /* Attempt to continue - heap exhausted */
}
```

---

## 11. Debugging Support

### 11.1 Runtime Statistics

```c
/* Task runtime statistics */
void print_task_stats(void)
{
    char buffer[512];
    vTaskGetRunTimeStats(buffer);
    log_info("Task Statistics:\n%s", buffer);
}

void print_task_list(void)
{
    char buffer[512];
    vTaskList(buffer);
    log_info("Task List:\n%s", buffer);
}
```

### 11.2 Trace Hooks

```c
/* Optional: Task switch tracing */
#if ENABLE_TRACE
#define traceTASK_SWITCHED_IN() trace_task_in(pxCurrentTCB)
#define traceTASK_SWITCHED_OUT() trace_task_out(pxCurrentTCB)
#endif
```

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
