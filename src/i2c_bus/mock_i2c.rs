#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MockI2cError {
    BusError,
}

impl embedded_hal::i2c::Error for MockI2cError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MockI2c {
    /// Simulated device register space.
    pub registers: [u8; 256],

    /// Register selected by the most recent write.
    current_register: u8,
}

impl MockI2c {
    pub const fn new() -> Self {
        Self { registers: [0; 256], current_register: 0 }
    }

    /// Set the value of a simulated register.
    #[allow(unused)]
    pub fn set_register(&mut self, register: u8, value: u8) {
        self.registers[register as usize] = value;
    }

    /// Set several consecutive registers.
    #[allow(unused)]
    pub fn set_registers(&mut self, register: u8, values: &[u8]) {
        let start = register as usize;
        let end = core::cmp::min(start + values.len(), self.registers.len());

        self.registers[start..end].copy_from_slice(&values[..end - start]);
    }

    /// Read the value of a simulated register.
    #[allow(unused)]
    pub fn register(&self, register: u8) -> u8 {
        self.registers[register as usize]
    }
}

impl embedded_hal::i2c::ErrorType for MockI2c {
    type Error = MockI2cError;
}

impl embedded_hal::i2c::I2c for MockI2c {
    fn read(&mut self, _address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let start = self.current_register as usize;
        let end = core::cmp::min(start + read.len(), self.registers.len());

        let len = end - start;

        read[..len].copy_from_slice(&self.registers[start..end]);

        if len < read.len() {
            read[len..].fill(0);
        }

        #[allow(clippy::cast_possible_truncation)]
        {
            self.current_register = self.current_register.wrapping_add(read.len() as u8);
        }

        Ok(())
    }

    fn write(&mut self, _address: u8, write: &[u8]) -> Result<(), Self::Error> {
        if write.is_empty() {
            return Ok(());
        }

        let register = write[0];
        self.current_register = register;

        // A single-byte write selects a register, which is the normal
        // I2C register-device convention.
        if write.len() > 1 {
            let start = register as usize;
            let data = &write[1..];
            let end = core::cmp::min(start + data.len(), self.registers.len());

            self.registers[start..end].copy_from_slice(&data[..end - start]);

            #[allow(clippy::cast_possible_truncation)]
            {
                self.current_register = register.wrapping_add(data.len() as u8);
            }
        }

        Ok(())
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.write(address, write)?;
        self.read(address, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                embedded_hal::i2c::Operation::Read(read) => {
                    self.read(address, read)?;
                }

                embedded_hal::i2c::Operation::Write(write) => {
                    self.write(address, write)?;
                }
            }
        }

        Ok(())
    }
}
