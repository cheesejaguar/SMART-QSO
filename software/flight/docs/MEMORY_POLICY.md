# Memory Allocation Policy
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-006
**Revision:** A
**Date:** 2026-01-02
**Reference:** JPL D-60411 (Power of Ten), NPR 7150.2

---

## 1. Purpose

This document defines the memory allocation policy for SMART-QSO flight software, ensuring deterministic behavior and compliance with JPL Power of Ten Rule 3 (no dynamic memory allocation after initialization).

## 2. Scope

This policy applies to all flight software executing on the OBC (STM32L4R5).

---

## 3. Memory Architecture

### 3.1 Memory Map

| Region | Start | Size | Usage |
|--------|-------|------|-------|
| Flash | 0x08000000 | 2 MB | Code, constants |
| SRAM1 | 0x20000000 | 192 KB | Stack, heap, data |
| SRAM2 | 0x20030000 | 64 KB | Critical data (TMR) |
| SRAM3 | 0x20040000 | 384 KB | Buffers, logs |
| Backup SRAM | 0x40024000 | 4 KB | Persistent state |

### 3.2 Memory Budget

| Category | Allocation | Used | Margin |
|----------|------------|------|--------|
| Code (Flash) | 512 KB | ~100 KB | 80% |
| Static Data | 64 KB | ~20 KB | 69% |
| Stack | 16 KB | ~4 KB | 75% |
| Heap (init only) | 32 KB | ~8 KB | 75% |
| Buffers | 128 KB | ~64 KB | 50% |
| Reserved | 64 KB | 0 KB | 100% |

---

## 4. Allocation Policy

### 4.1 Permitted Allocations

| Phase | malloc/calloc | Static | Stack |
|-------|:-------------:|:------:|:-----:|
| Initialization | Permitted | Permitted | Permitted |
| Runtime | **PROHIBITED** | N/A | Permitted |
| Shutdown | **PROHIBITED** | N/A | Permitted |

### 4.2 Rule: No Dynamic Allocation After Init

Per JPL Power of Ten Rule 3:
- All `malloc()`, `calloc()`, `realloc()` calls must complete before `main()` loop entry
- No `free()` calls after initialization
- No dynamic memory allocation in interrupt handlers

### 4.3 Enforcement

```c
/* Build-time enforcement */
#ifdef FLIGHT_BUILD
  #define malloc(x)  COMPILE_ERROR_NO_MALLOC_IN_FLIGHT
  #define calloc(x,y) COMPILE_ERROR_NO_CALLOC_IN_FLIGHT
  #define realloc(x,y) COMPILE_ERROR_NO_REALLOC_IN_FLIGHT
  #define free(x)    COMPILE_ERROR_NO_FREE_IN_FLIGHT
#endif

/* Link-time enforcement */
/* Use --wrap=malloc linker flag to catch accidental allocations */
```

---

## 5. Buffer Management

### 5.1 Static Buffer Pools

All buffers are statically allocated at compile time:

```c
/* Telemetry buffer pool */
static uint8_t s_tlm_buffers[TLM_BUFFER_COUNT][TLM_BUFFER_SIZE];
static bool s_tlm_buffer_used[TLM_BUFFER_COUNT];

/* Beacon buffer pool */
static char s_beacon_buffers[BEACON_BUFFER_COUNT][BEACON_MAX_SIZE];

/* UART RX/TX buffers */
static uint8_t s_uart_rx_buffer[UART_PORT_COUNT][UART_RX_SIZE];
static uint8_t s_uart_tx_buffer[UART_PORT_COUNT][UART_TX_SIZE];
```

### 5.2 Buffer Allocation API

```c
/* Get buffer from pool (non-blocking) */
void* buffer_pool_alloc(BufferPool_t* pool);

/* Return buffer to pool */
void buffer_pool_free(BufferPool_t* pool, void* buffer);

/* Check pool status */
size_t buffer_pool_available(BufferPool_t* pool);
```

### 5.3 Buffer Sizes

| Buffer | Size | Count | Total |
|--------|------|-------|-------|
| Telemetry | 256 B | 8 | 2 KB |
| Beacon | 256 B | 4 | 1 KB |
| UART RX | 512 B | 3 | 1.5 KB |
| UART TX | 512 B | 3 | 1.5 KB |
| Sensor Data | 64 B | 16 | 1 KB |
| Fault Log | 32 B | 64 | 2 KB |

---

## 6. Stack Management

### 6.1 Stack Allocation

| Task/Context | Stack Size | High Watermark Target |
|--------------|------------|----------------------|
| Main task | 4 KB | <75% |
| Sensor task | 1 KB | <75% |
| Beacon task | 2 KB | <75% |
| ADCS task | 2 KB | <75% |
| Fault handler | 1 KB | <75% |
| ISR stack | 1 KB | <50% |

### 6.2 Stack Protection

```c
/* Stack canary for overflow detection */
#define STACK_CANARY_VALUE 0xDEADBEEF

/* Stack guard regions */
/* MPU configured to fault on stack overflow */
```

### 6.3 Stack Monitoring

- Stack high watermark checked at runtime
- Warning logged if usage exceeds 75%
- Fault triggered if usage exceeds 90%

---

## 7. Critical Data Storage

### 7.1 Triple Modular Redundancy (TMR)

Critical configuration stored in triplicate:

```c
typedef struct {
    uint32_t copy_a;
    uint32_t copy_b;
    uint32_t copy_c;
} TmrValue_t;

uint32_t tmr_read(const TmrValue_t* tmr);
void tmr_write(TmrValue_t* tmr, uint32_t value);
```

### 7.2 TMR Data Items

| Item | Size | Location |
|------|------|----------|
| Power mode | 4 B x 3 | SRAM2 |
| Beacon interval | 4 B x 3 | SRAM2 |
| Boot count | 4 B x 3 | Backup SRAM |
| Mission time | 8 B x 3 | Backup SRAM |
| Configuration CRC | 4 B x 3 | SRAM2 |

---

## 8. Persistent Storage

### 8.1 Non-Volatile Storage Map

| Item | Address | Size | Protection |
|------|---------|------|------------|
| Boot counter | 0x0000 | 16 B | TMR + CRC |
| Configuration | 0x0100 | 512 B | CRC32 |
| Mission data | 0x0400 | 1 KB | CRC32 |
| Fault log | 0x0800 | 2 KB | CRC32 |
| Reserved | 0x1000 | 4 KB | - |

### 8.2 Write Policy

- Minimize write cycles (flash wear)
- Batch writes where possible
- Verify after write
- Log write failures

---

## 9. Memory Safety

### 9.1 Bounds Checking

All array accesses validated:

```c
/* Safe array access macro */
#define SAFE_ARRAY_ACCESS(arr, idx, default_val) \
    (((idx) < ARRAY_SIZE(arr)) ? (arr)[idx] : (default_val))

/* String copy with bounds */
void safe_strncpy(char* dest, size_t dest_size, const char* src);
```

### 9.2 Null Pointer Checks

All pointer parameters validated:

```c
SmartQsoResult_t function(const Data_t* data) {
    if (data == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    /* ... */
}
```

### 9.3 Integer Overflow Protection

```c
/* Safe addition with overflow check */
bool safe_add_u32(uint32_t a, uint32_t b, uint32_t* result);

/* Safe multiplication with overflow check */
bool safe_mul_u32(uint32_t a, uint32_t b, uint32_t* result);
```

---

## 10. Memory Scrubbing

### 10.1 SRAM Scrubbing

- Periodic read-modify-write of SRAM regions
- ECC correction triggered on read
- Scrub rate: 1 MB/hour (full SRAM every ~40 min)

### 10.2 Scrub Implementation

```c
void memory_scrub_task(void) {
    static uint32_t scrub_addr = SRAM_START;

    /* Read location (triggers ECC correction) */
    volatile uint32_t value = *(uint32_t*)scrub_addr;

    /* Write back (refreshes cell) */
    *(uint32_t*)scrub_addr = value;

    /* Advance to next location */
    scrub_addr += 4;
    if (scrub_addr >= SRAM_END) {
        scrub_addr = SRAM_START;
    }
}
```

---

## 11. Verification

### 11.1 Static Analysis

- Cppcheck: Detect memory leaks, buffer overflows
- PC-lint: MISRA Rule 21.3 (no stdlib memory functions)

### 11.2 Runtime Checks

- Stack canary verification
- Heap corruption detection (during init only)
- Buffer pool integrity checks

### 11.3 Testing

- Memory stress tests during integration
- Stack overflow injection tests
- Buffer exhaustion tests

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
