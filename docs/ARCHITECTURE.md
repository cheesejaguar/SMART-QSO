# SMART-QSO Satellite Architecture
## Detailed Hardware and Software Specification for NASA PDR

### Executive Summary

SMART-QSO is a 1U CubeSat mission designed to demonstrate advanced AI capabilities in space while providing amateur radio transponder services. The satellite features a novel architecture that integrates a high-performance AI payload (Jetson Orin Nano Super) within the constraints of a 1U form factor, supported by an ultra-low-power onboard computer and intelligent power management system.

---

## 1.0 System Architecture Overview

### 1.1 Mission Architecture
SMART-QSO employs a **hybrid architecture** combining traditional CubeSat subsystems with advanced AI capabilities:

- **Primary Mission**: AI-powered amateur radio transponder service
- **Secondary Mission**: Technology demonstration of AI in constrained space environments
- **Architecture Philosophy**: AI-first design with graceful degradation to conventional operations

### 1.2 System Block Diagram
```
┌─────────────────────────────────────────────────────────────┐
│                    SMART-QSO 1U CubeSat                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Solar     │  │   Solar     │  │    Deployable       │ │
│  │   Panel 1   │  │   Panel 2   │  │    Solar Wings      │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    AI Payload                           │ │
│  │              Jetson Orin Nano Super                     │ │
│  │              (Power-gated, duty-cycled)                │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │    OBC      │  │     EPS     │  │        ADCS         │ │
│  │  Ambiq/STM  │  │   MPPT +    │  │    Magnetorquers    │ │
│  │   Apollo4   │  │   Battery   │  │   + Sun Sensors     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                  Communication                          │ │
│  │              UHF/VHF Transponder                        │ │
│  │              + Beacon (AX.25)                          │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 2.0 Hardware Architecture

### 2.1 Mechanical Structure and Form Factor

#### 2.1.1 CubeSat Standard Compliance
- **Form Factor**: 1U CubeSat (100mm × 100mm × 113.5mm)
- **Mass**: ≤ 1.33 kg (per Cal Poly CubeSat Design Specification)
- **Material**: Aluminum 6061-T6 with thermal treatment
- **Deployment Interface**: NanoRacks CubeSat Deployer (NRCSD) compliant

#### 2.1.2 Structural Design
- **Primary Structure**: Monocoque design with internal ribbing
- **Wall Thickness**: 2.0 mm minimum for structural integrity
- **Corner Rails**: Standard 12.7 mm × 12.7 mm with deployment springs
- **Thermal Interface**: Black anodized finish for thermal control

#### 2.1.3 Deployable Solar Wings
- **Configuration**: 2 deployable wings, 1 per side
- **Wing Dimensions**: 100mm × 50mm × 2mm
- **Deployment Mechanism**: Spring-loaded with retention pins
- **Hinge Design**: Frictionless bearings with deployment stops
- **Stowed Position**: Parallel to CubeSat body
- **Deployed Position**: 90° from body, locked in place

### 2.2 Electrical Power Subsystem (EPS)

#### 2.2.1 Solar Array Configuration
- **Total Solar Cells**: 8 cells (4 per wing)
- **Cell Type**: Triple-junction GaAs (30% efficiency)
- **Cell Dimensions**: 25mm × 25mm
- **Cell Configuration**: 2S4P per wing, 4S4P total
- **Peak Power**: 4.8W at beginning of life (BOL)
- **End of Life Power**: 4.2W after 2 years

#### 2.2.2 Power Management
- **MPPT Controller**: GomSpace NanoPower MPPT
- **Input Voltage Range**: 6.0V - 18.0V
- **MPPT Efficiency**: >95% at maximum power point
- **Output Voltage**: 3.3V, 5.0V, 12.0V regulated rails

#### 2.2.3 Energy Storage
- **Battery Type**: 4S Li-ion 18650 pack
- **Total Capacity**: 50 Wh (13.5 Ah at 3.7V nominal)
- **Voltage Range**: 12.0V (0% SoC) to 14.8V (100% SoC)
- **Cell Configuration**: 4 cells in series
- **Battery Management**: ISIS iBMS with overcharge/overdischarge protection
- **Cycle Life**: >500 cycles to 80% capacity

#### 2.2.4 Power Distribution
- **Power Distribution Unit**: GomSpace NanoPower PDU
- **Switched Rails**: 3.3V, 5.0V, 12.0V with individual switching
- **Current Monitoring**: Hall-effect sensors on each rail
- **Fault Protection**: Overcurrent, overvoltage, undervoltage protection
- **Load Shedding**: Intelligent power management with priority-based shedding

### 2.3 Attitude Determination and Control System (ADCS)

#### 2.3.1 Sensors
- **Magnetometer**: 3-axis fluxgate magnetometer (0.1 μT resolution)
- **Sun Sensors**: 6 coarse sun sensors (CSS) for attitude determination
- **Temperature Sensors**: 4 temperature sensors for thermal monitoring

#### 2.3.2 Actuators
- **Magnetorquers**: 3-axis magnetorquer coils
- **Coil Configuration**: X, Y, Z axes with 0.1 A⋅m² dipole moment
- **Current Rating**: 100 mA maximum per axis
- **Power Consumption**: 0.5W per axis when active

#### 2.3.3 Control Algorithms
- **Detumble Mode**: B-dot algorithm for initial detumbling
- **Sun Pointing**: Sun-tracking algorithm for solar panel optimization
- **Control Frequency**: 1 Hz control loop
- **Pointing Accuracy**: ±10° in sun-pointing mode

### 2.4 Communication Subsystem

#### 2.4.1 Transponder System
- **Frequency Bands**: UHF (435-438 MHz), VHF (144-146 MHz)
- **Transmitter Power**: 1W maximum (adjustable)
- **Receiver Sensitivity**: -120 dBm
- **Modulation**: FM for voice, FSK for data
- **Bandwidth**: 25 kHz channels
- **Protocol**: AX.25 packet radio

#### 2.4.2 Beacon System
- **Frequency**: 145.825 MHz (primary), 437.425 MHz (secondary)
- **Transmission Interval**: 120 seconds (configurable)
- **Message Format**: AX.25 packet with telemetry data
- **Power Consumption**: 0.1W idle, 0.5W during transmission

#### 2.4.3 Antenna System
- **UHF Antenna**: 1/4 wave monopole, deployable
- **VHF Antenna**: 1/4 wave monopole, deployable
- **Antenna Deployment**: Spring-loaded with retention mechanism
- **Gain**: 2 dBi typical
- **Polarization**: Linear, vertical

### 2.5 AI Payload - Jetson Orin Nano Super

#### 2.5.1 Hardware Specifications
- **Processor**: NVIDIA Jetson Orin Nano Super
- **CPU**: 8-core ARM Cortex-A78AE (2.0 GHz)
- **GPU**: 2048 CUDA cores, 64 Tensor cores
- **AI Performance**: 20 TOPS (Trillion Operations Per Second)
- **Memory**: 8GB LPDDR5, 32GB eMMC storage
- **Power Consumption**: 5W typical, 15W peak
- **Thermal Design**: Passive cooling with thermal interface to structure

#### 2.5.2 Power Management
- **Power Gating**: Individual rail control for power management
- **DVFS**: Dynamic voltage and frequency scaling
- **Core Parking**: Ability to disable unused CPU cores
- **GPU Scaling**: Dynamic GPU power management
- **Sleep Modes**: Multiple low-power states

#### 2.5.3 Thermal Management
- **Heat Dissipation**: Conduction through mounting interface
- **Thermal Interface**: High-conductivity thermal pad
- **Temperature Monitoring**: Onboard temperature sensors
- **Thermal Control**: Duty cycling based on temperature

### 2.6 Onboard Computer (OBC)

#### 2.6.1 Primary OBC
- **Processor**: Ambiq Apollo4 Blue (ARM Cortex-M4F)
- **Clock Speed**: 96 MHz maximum
- **Memory**: 1MB SRAM, 2MB flash
- **Power Consumption**: <10 mW typical
- **Operating System**: FreeRTOS

#### 2.6.2 Secondary OBC
- **Processor**: STM32L4 series (ARM Cortex-M4)
- **Clock Speed**: 80 MHz maximum
- **Memory**: 640KB SRAM, 1MB flash
- **Power Consumption**: <50 mW typical
- **Operating System**: FreeRTOS

#### 2.6.3 OBC Functions
- **System Control**: Overall satellite management
- **Telemetry Collection**: Sensor data acquisition and formatting
- **Command Processing**: Ground command interpretation and execution
- **Fault Management**: System health monitoring and recovery
- **Data Storage**: Mission data logging and storage

### 2.7 Data Storage and Management

#### 2.7.1 Primary Storage
- **FRAM**: 4MB ferroelectric RAM for critical data
- **Purpose**: System state, configuration parameters, mission data
- **Write Endurance**: 10^14 write cycles
- **Data Retention**: 10 years at 85°C

#### 2.7.2 Secondary Storage
- **MicroSD Card**: 32GB class 10 for log storage
- **NAND Flash**: 16GB for mission data and AI models
- **Data Organization**: Time-stamped logs and mission data
- **Redundancy**: Critical data stored in multiple locations

---

## 3.0 Software Architecture

### 3.1 System Software Architecture

#### 3.1.1 Operating System Layer
- **Primary OBC**: FreeRTOS v10.4.3
- **Secondary OBC**: FreeRTOS v10.4.3
- **AI Payload**: Ubuntu 20.04 LTS with real-time kernel patches
- **Inter-Process Communication**: Shared memory and message queues

#### 3.1.2 Middleware Layer
- **Communication Stack**: AX.25 protocol implementation
- **File System**: FAT32 for microSD, ext4 for NAND flash
- **Database**: SQLite for structured data storage
- **Network Stack**: TCP/IP for AI payload communication

#### 3.1.3 Application Layer
- **Mission Control**: Main mission state machine
- **Power Manager**: Intelligent power scheduling and management
- **AI Agent Manager**: AI payload coordination and control
- **Communication Manager**: Transponder and beacon control

### 3.2 AI Software Architecture

#### 3.2.1 AI Framework
- **Deep Learning**: PyTorch 1.12 with TensorRT optimization
- **Computer Vision**: OpenCV 4.6 for image processing
- **Natural Language**: Hugging Face Transformers for text processing
- **Optimization**: ONNX Runtime for model optimization

#### 3.2.2 AI Models
- **QSO Priority Model**: Tiny CNN (~1MB) for call sign prioritization
- **Adaptive Transponder**: Reinforcement learning for channel optimization
- **Smart Beacon**: TinyLM (~1MB) for intelligent beacon messages
- **Model Updates**: Over-the-air updates via ground station

#### 3.2.3 AI Workflow
- **Data Ingestion**: Real-time communication data processing
- **Inference**: Onboard AI model execution
- **Learning**: Incremental model updates based on usage patterns
- **Optimization**: Model compression and quantization

### 3.3 Power Management Software

#### 3.3.1 Power Forecasting
- **Solar Prediction**: Orbit-based solar illumination modeling
- **Load Prediction**: Mission phase and AI payload usage modeling
- **Battery Modeling**: Coulomb counting with temperature compensation
- **Power Budgeting**: Real-time power allocation and optimization

#### 3.3.2 Intelligent Scheduling
- **AI Payload Scheduling**: Duty cycling based on power availability
- **Communication Scheduling**: Beacon and transponder timing optimization
- **Thermal Management**: Temperature-based power scheduling
- **Priority Management**: Critical vs. non-critical load management

#### 3.3.3 Fault Management
- **Overcurrent Protection**: Automatic load shedding
- **Undervoltage Protection**: Battery protection and system shutdown
- **Thermal Protection**: Temperature-based power reduction
- **Recovery Procedures**: Automatic system recovery and restart

### 3.4 Communication Software

#### 3.4.1 Protocol Implementation
- **AX.25**: Full AX.25 protocol stack implementation
- **Packet Radio**: APRS and packet data support
- **Voice Support**: FM voice transmission and reception
- **Data Support**: Binary data transmission and reception

#### 3.4.2 Beacon System
- **Telemetry Format**: Standardized telemetry packet structure
- **Data Content**: System health, mission status, AI payload status
- **Transmission Timing**: Configurable intervals and conditions
- **Message Prioritization**: Critical vs. routine message handling

#### 3.4.3 Transponder Control
- **Channel Management**: Automatic channel selection and optimization
- **Power Control**: Dynamic power adjustment based on conditions
- **Interference Avoidance**: AI-powered channel optimization
- **Quality Monitoring**: Signal quality and performance metrics

### 3.5 Data Management Software

#### 3.5.1 Data Collection
- **Sensor Data**: Continuous monitoring of all subsystem sensors
- **Mission Data**: Communication logs and AI payload data
- **System Data**: Performance metrics and health indicators
- **Timestamping**: Precise time synchronization for all data

#### 3.5.2 Data Storage
- **Data Organization**: Hierarchical file system organization
- **Compression**: Lossless compression for log data
- **Encryption**: Secure storage for sensitive mission data
- **Backup**: Redundant storage and data protection

#### 3.5.3 Data Transmission
- **Downlink Prioritization**: Critical data transmission priority
- **Compression**: Onboard data compression for efficient transmission
- **Error Correction**: Forward error correction for reliable transmission
- **Acknowledgment**: Ground station acknowledgment and retransmission

---

## 4.0 System Integration and Interfaces

### 4.1 Electrical Interfaces

#### 4.1.1 Power Distribution
- **Main Power Bus**: 12V primary bus with 3.3V and 5V regulated rails
- **Power Switching**: Individual rail control with current monitoring
- **Grounding**: Single-point ground system with isolation
- **EMI Protection**: Filtering and shielding for sensitive electronics

#### 4.1.2 Communication Interfaces
- **UART**: Serial communication between subsystems
- **SPI**: High-speed data transfer for sensors and storage
- **I2C**: Low-speed control and configuration
- **CAN Bus**: Redundant communication for critical systems

#### 4.1.3 Data Interfaces
- **USB**: AI payload interface and data transfer
- **Ethernet**: High-speed data transfer for AI payload
- **GPIO**: Digital I/O for control and monitoring
- **ADC**: Analog sensor interfaces

### 4.2 Mechanical Interfaces

#### 4.2.1 Structural Integration
- **Mounting Points**: Standardized mounting for all subsystems
- **Thermal Interface**: Conductive thermal paths to structure
- **Vibration Isolation**: Shock mounts for sensitive electronics
- **Accessibility**: Service access for testing and maintenance

#### 4.2.2 Deployment Mechanisms
- **Solar Panel Deployment**: Spring-loaded with retention pins
- **Antenna Deployment**: Spring-loaded with retention mechanism
- **Deployment Sensors**: Position and status monitoring
- **Deployment Control**: Automatic and manual deployment options

### 4.3 Software Interfaces

#### 4.3.1 Application Programming Interfaces (APIs)
- **Power Management API**: Power control and monitoring interface
- **Communication API**: Transponder and beacon control interface
- **AI Payload API**: AI model execution and data interface
- **System Control API**: Overall system management interface

#### 4.3.2 Data Interfaces
- **Telemetry Interface**: Standardized telemetry data format
- **Command Interface**: Ground command processing interface
- **Configuration Interface**: System parameter configuration interface
- **Diagnostic Interface**: System health and diagnostic interface

---

## 5.0 Reliability and Fault Tolerance

### 5.1 System Reliability

#### 5.1.1 Design Reliability
- **Component Selection**: Space-grade or high-reliability commercial components
- **Redundancy**: Critical system redundancy where feasible
- **Derating**: Conservative component derating for reliability
- **Testing**: Comprehensive component and system testing

#### 5.1.2 Mission Reliability
- **Mission Duration**: 2-year design life with 5-year goal
- **Reliability Target**: 90% probability of mission success
- **Failure Modes**: Comprehensive failure mode analysis
- **Recovery Procedures**: Automatic and manual recovery procedures

### 5.2 Fault Tolerance

#### 5.2.1 Graceful Degradation
- **AI Payload Failure**: Fallback to conventional transponder operation
- **Power System Failure**: Load shedding and minimal operation mode
- **Communication Failure**: Beacon-only operation mode
- **OBC Failure**: Secondary OBC takeover

#### 5.2.2 Fault Detection and Recovery
- **Health Monitoring**: Continuous system health monitoring
- **Fault Detection**: Automatic fault detection and isolation
- **Recovery Procedures**: Automatic recovery and restart procedures
- **Ground Intervention**: Manual recovery procedures via ground commands

---

## 6.0 Testing and Verification

### 6.1 Component Testing

#### 6.1.1 Environmental Testing
- **Thermal Testing**: -40°C to +85°C operational range
- **Vibration Testing**: Launch vehicle vibration environment
- **Thermal Vacuum**: Space environment simulation
- **Radiation Testing**: Total ionizing dose and single event effects

#### 6.1.2 Functional Testing
- **Performance Testing**: All performance specifications verification
- **Interface Testing**: Subsystem interface compatibility
- **Power Testing**: Power consumption and efficiency verification
- **Communication Testing**: Communication system performance

### 6.2 System Testing

#### 6.2.1 Integration Testing
- **Subsystem Integration**: Individual subsystem integration testing
- **System Integration**: Complete system integration testing
- **Interface Verification**: All interface compatibility verification
- **Performance Verification**: System-level performance verification

#### 6.2.2 Mission Simulation
- **Orbit Simulation**: Mission orbit and environment simulation
- **Mission Phases**: All mission phase simulation and testing
- **Failure Scenarios**: Failure mode and recovery testing
- **End-to-End Testing**: Complete mission simulation

---

## 7.0 Mission Operations

### 7.1 Launch and Early Operations

#### 7.1.1 Launch Phase
- **Deployment**: NRCSD deployment and initial separation
- **Initialization**: System power-up and initialization sequence
- **Deployment Verification**: Solar panel and antenna deployment verification
- **Initial Communication**: First beacon transmission and ground contact

#### 7.1.2 Commissioning Phase
- **System Checkout**: Comprehensive system health verification
- **AI Payload Activation**: Gradual AI payload activation and testing
- **Performance Verification**: All system performance verification
- **Mission Readiness**: Mission readiness declaration

### 7.2 Normal Operations

#### 7.2.1 Daily Operations
- **Health Monitoring**: Continuous system health monitoring
- **AI Payload Operations**: AI model execution and learning
- **Communication Services**: Transponder and beacon services
- **Data Management**: Mission data collection and transmission

#### 7.2.2 Mission Phases
- **Commissioning**: System checkout and AI activation
- **Normal Operations**: Full mission operations
- **Extended Operations**: Extended mission operations
- **End of Mission**: Mission termination and disposal

### 7.3 Anomaly Response

#### 7.3.1 Anomaly Detection
- **Automatic Detection**: Onboard anomaly detection and classification
- **Ground Analysis**: Ground station anomaly analysis and response
- **Escalation Procedures**: Automatic and manual escalation procedures
- **Recovery Actions**: Automatic and manual recovery actions

---

## 8.0 Ground Segment

### 8.1 Ground Station Network

#### 8.1.1 Primary Ground Station
- **Location**: Mission control center
- **Equipment**: UHF/VHF transceivers and tracking equipment
- **Capabilities**: Full duplex communication and tracking
- **Availability**: 24/7 operation with redundancy

#### 8.1.2 Secondary Ground Stations
- **Amateur Radio Network**: Global amateur radio operator network
- **Educational Institutions**: University and educational institution stations
- **Individual Operators**: Individual amateur radio operators
- **Data Collection**: Distributed data collection and reporting

### 8.2 Mission Control Software

#### 8.2.1 Command and Control
- **Command Generation**: Mission command generation and validation
- **Command Uplink**: Secure command transmission to satellite
- **Command Verification**: Command execution verification
- **Command History**: Complete command history and audit trail

#### 8.2.2 Telemetry Processing
- **Data Reception**: Telemetry data reception and processing
- **Data Analysis**: Real-time data analysis and visualization
- **Data Storage**: Long-term data storage and archiving
- **Data Distribution**: Data distribution to stakeholders

---

## 9.0 Cost and Schedule

### 9.1 Cost Breakdown

#### 9.1.1 Hardware Costs
- **AI Payload**: $2,500 (Jetson Orin Nano Super)
- **OBC and Subsystems**: $3,000 (EPS, ADCS, Communication)
- **Structure and Mechanisms**: $1,500 (Mechanical components)
- **Testing and Integration**: $2,000 (Environmental testing)

#### 9.1.2 Software and Development
- **Software Development**: $5,000 (System and AI software)
- **Testing and Verification**: $2,500 (Software testing)
- **Documentation**: $1,000 (Technical documentation)

#### 9.1.3 Operations and Launch
- **Launch Services**: $100,000 (NanoRacks launch)
- **Ground Operations**: $10,000 (Ground station operations)
- **Insurance**: $15,000 (Launch and mission insurance)

**Total Mission Cost**: $142,500

### 9.2 Schedule

#### 9.2.1 Development Phase
- **Phase 1**: Requirements and design (3 months)
- **Phase 2**: Hardware development (6 months)
- **Phase 3**: Software development (6 months)
- **Phase 4**: Integration and testing (3 months)

#### 9.2.2 Launch and Operations
- **Launch**: Month 18
- **Commissioning**: Months 18-19
- **Normal Operations**: Months 19-42
- **Extended Operations**: Months 42-66

**Total Development Time**: 18 months

---

## 10.0 Risk Assessment and Mitigation

### 10.1 Technical Risks

#### 10.1.1 AI Payload Risks
- **Power Consumption**: Risk of exceeding power budget
- **Mitigation**: Advanced power management and duty cycling
- **Thermal Management**: Risk of thermal overload
- **Mitigation**: Integrated thermal management and monitoring

#### 10.1.2 Integration Risks
- **System Complexity**: Risk of integration challenges
- **Mitigation**: Modular design and comprehensive testing
- **Interface Compatibility**: Risk of interface incompatibilities
- **Mitigation**: Standardized interfaces and thorough testing

### 10.2 Mission Risks

#### 10.2.1 Launch Risks
- **Launch Failure**: Risk of launch vehicle failure
- **Mitigation**: Launch insurance and backup launch opportunities
- **Deployment Failure**: Risk of deployment mechanism failure
- **Mitigation**: Redundant deployment mechanisms and testing

#### 10.2.2 Operations Risks
- **Communication Loss**: Risk of communication system failure
- **Mitigation**: Redundant communication systems and ground stations
- **Power System Failure**: Risk of power system malfunction
- **Mitigation**: Redundant power systems and load shedding

---

## 11.0 Conclusion

SMART-QSO represents a significant advancement in CubeSat technology, demonstrating that sophisticated AI capabilities can be successfully integrated into the constrained 1U form factor. The mission's innovative architecture combines advanced AI processing with traditional CubeSat subsystems, creating a new paradigm for small satellite missions.

The detailed hardware and software architecture provides a solid foundation for mission success, with comprehensive consideration of power management, thermal control, reliability, and fault tolerance. The mission's focus on amateur radio outreach and community engagement adds a unique dimension that distinguishes it from previous AI CubeSat missions.

SMART-QSO's success will establish new benchmarks for AI integration in small satellites and demonstrate the potential for sophisticated AI capabilities in constrained space environments, while providing valuable services to the amateur radio community.

---

## 12.0 Appendices

### Appendix A: Detailed Component Specifications
- Complete component datasheets and specifications
- Interface definitions and pin assignments
- Performance characteristics and operating parameters

### Appendix B: Software Architecture Details
- Detailed software module specifications
- API definitions and interface specifications
- Data flow diagrams and state machines

### Appendix C: Test Plans and Procedures
- Comprehensive test plans for all subsystems
- Test procedures and acceptance criteria
- Environmental testing requirements and procedures

### Appendix D: Operations Manual
- Detailed operations procedures and protocols
- Anomaly response procedures
- Ground station operations and maintenance

---

*Document Version: 1.0*  
*Date: [Current Date]*  
*Prepared by: [Name]*  
*Technical Review: [Name]*  
*Management Approval: [Name]*  
*NASA PDR Approval: [Name]*
