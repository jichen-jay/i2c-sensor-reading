## ESP32 Rust esp-hal Migration Issues - Detailed Reference Guide

### **❌ Confirmed Breaking Error Patterns:**

#### **1. esp-hal 1.0.0-beta.1 Linker Failures:**
```
rust-lld: error: undefined symbol: rom_i2c_writeReg
rust-lld: error: undefined symbol: ets_update_cpu_frequency  
rust-lld: error: undefined symbol: ets_delay_us
rust-lld: error: undefined symbol: _bss_start
rust-lld: error: undefined symbol: __stack_chk_guard
rust-lld: error: undefined symbol: WIFI_MAC
rust-lld: error: undefined symbol: BT_MAC
```
**Cause**: esp-hal 1.0.0-beta.1 has incomplete ROM function bindings
**Solution**: AVOID - use esp-hal 0.23.x instead

#### **2. Runtime Conflicts:**
```
rust-lld: error: duplicate symbol: _start
>>> defined at riscv_rt.4db9d2381488c96d-cgu.0
>>> defined at esp_riscv_rt.cfed5ad644c2aba5-cgu.0
```
**Cause**: Adding `riscv-rt = "0.12"` alongside esp-hal creates duplicate symbols
**Solution**: Remove riscv-rt, let esp-hal provide its own runtime

#### **3. embedded-hal Version Mismatch Errors:**
```
error[E0277]: the trait bound `I2cProxy<'_, NullMutex<Result<..., ...>>>: I2c` is not satisfied
note: there are multiple different versions of crate `embedded_hal` in the dependency graph
```
**Cause**: Mixing embedded-hal 0.2.x drivers with embedded-hal 1.0.x HAL
**Examples**:
- `shtcx = "1.0.0"` (uses embedded-hal 0.2.x) + `esp-hal` 1.0 (uses embedded-hal 1.0.x)
- `shared-bus = "0.3.1"` (embedded-hal 0.2.x) + modern HAL

#### **4. Bus Sharing Feature Errors:**
```
error: failed to select a version for `embedded-hal-bus`.
the package depends on `embedded-hal-bus`, with features: `critical-section` but `embedded-hal-bus` does not have these features.
```
**Cause**: `embedded-hal-bus = { version = "0.2.0", features = ["critical-section"] }` - feature doesn't exist
**Solution**: Use separate `critical-section = "1.1"` dependency

#### **5. esp-hal API Changes Between Versions:**
**0.23.x API:**
```rust
let peripherals = Peripherals::take();
let system = peripherals.SYSTEM.split();
let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
let i2c = I2C::new(peripherals.I2C0, sda_pin, scl_pin, 400.kHz(), &clocks);
```

**1.0.x API:**
```rust  
let peripherals = esp_hal::init(esp_hal::Config::default());
let i2c = I2c::new(peripherals.I2C0, i2c_config).unwrap();
```

#### **6. Entry Point Inconsistencies:**
```
error[E0433]: failed to resolve: could not find `entry` in `esp_hal`
```
**Patterns that fail**:
- `#[esp_hal::entry]` - doesn't exist in some versions
- `#[main]` - requires specific features
- `#[entry]` from riscv-rt - conflicts with ESP runtime

### **✅ Confirmed Working Patterns:**

#### **1. embedded-hal-compat Bridge (VERIFIED WORKING):**
```toml
embedded-hal-compat = "0.13.0"
shtcx = "1.0.0"  # 0.2.x driver
esp-hal = { version = "0.23", features = ["esp32c3"] }  # 1.0.x HAL
```
```rust
use embedded_hal_compat::ReverseCompat;
let i2c_compat = i2c.reverse();  // Bridge 1.0.x HAL to 0.2.x driver
let mut sht = shtcx::shtc3(i2c_compat);
```

#### **2. Pure embedded-hal 1.0.x Stack (VERIFIED WORKING):**
```toml
embedded-hal-bus = "0.2.0"
critical-section = "1.1"  # Separate dependency
shtc3 = "0.1.0"  # Native 1.0.x driver instead of shtcx
```
```rust
use embedded_hal_bus::i2c::CriticalSectionDevice;
use critical_section::Mutex;
let bus = Mutex::new(RefCell::new(i2c));
let device = CriticalSectionDevice::new(&bus);
```

#### **3. Version Pinning Strategy (TESTED):**
```toml
# WORKING COMBINATION:
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
embedded-hal = "1.0.0"
embedded-hal-compat = "0.13.0"
shared-bus = "0.3.1"  # For 0.2.x compatibility only
```

### **🔧 Specific Error → Solution Mappings:**

| Error Pattern | Root Cause | Immediate Fix |
|---------------|------------|---------------|
| `undefined symbol: rom_i2c_writeReg` | esp-hal 1.0.0-beta.1 broken | Use esp-hal 0.23.x |
| `duplicate symbol: _start` | Runtime conflict | Remove riscv-rt dependency |
| `trait bound...I2c...not satisfied` | embedded-hal version mismatch | Add embedded-hal-compat |
| `critical-section` feature missing | Wrong embedded-hal-bus usage | Use separate critical-section crate |
| `could not find entry in esp_hal` | Entry point API changes | Use `#[no_mangle] pub extern "C" fn main()` |

### **📋 Debugging Commands:**
```bash
# Check dependency tree for version conflicts:
cargo tree | grep embedded-hal

# Check for duplicate symbols:
cargo build --verbose 2>&1 | grep "duplicate symbol"

# Test with minimal dependencies:
cargo build -Z build-std=core --target riscv32imc-unknown-none-elf
```

### **🎯 LLM Usage Notes:**
- When seeing linker errors with ESP ROM functions → immediate red flag for esp-hal 1.0.0-beta.1
- When seeing trait bound errors with I2c → check for embedded-hal version mixing
- When seeing duplicate _start symbols → runtime dependency conflict
- Always start with proven working example from esp-rs/esp-hal repository
- Pin ALL dependency versions when something works