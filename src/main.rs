// Ref: rust in action

// CHIP-8 Emulator.

// Decoding CHIP-8 opcodes.
// they're u16 values made up of 4 nibbles (half a byte / 4 bits)
// rust has no 4 bit type so we just wrap them all together in a 16bit type and then awkwardly
// filter out the bits we dont care about to access each individual nibble

// CHIP-8 breaks opcodes down into 2 bytes, high byte, low byte, then nibbles, high nibble, low nibble

// 0xAB12
// high byte = AB
  // high nibble A
  // low nibble B
// low byte = 12
  // high nibble 1
  // low nibble 2

// All CHIP-8 opcodes are U16 values, defined by who makes the architecture
struct CPU {
    // Moved now to 16 registers. Means that a single hex num (0 to F) can address these,
    // let's all opcodes be compactly represented as u16 values.
    registers: [u8; 16],
    // Usually called 'program counter' but this naming makes it obvious
    position_in_memory: usize, // diverges from original spec, but rust lets us use this for indexing
    // 0x1000 is hex for 4096 (4kb), the amount of bytes of RAM a CHIP-8 had.
    // The chip-8 usize equiv basically, only 2^12 (12 bits = 4096)
    // In original spec, the first 512 bytes (0x100) are reserved for the system, others are for programs
    memory: [u8; 0x1000]
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        // combine 2 u8 into a single u16
        let p = self.position_in_memory;
        let op_byte1 = self.memory[p] as u16;
        let op_byte2 = self.memory[p + 1] as u16;

        // to create a u16 opcode, combine two values from memory with logical OR
        // they need to be cast as u16 to start with; otherwise,
        // the left shift sets all of the bits to 0
        // left shift to ignore the right most 8 bits, we're adding op_byte2 to get those.
        op_byte1 << 8 | op_byte2
    }

    /// Main CPU loop
    /// 1. Read u16 opcode from values in memory (2 u8 values, the high byte and low byte)
    /// 2. Decodes instructions
    /// 3. Matches decoded instructions to known opcodes
    /// 4. dispatches execution of the operation to a specific function
    fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();

            // we've read and loaded the instruction from memory; point to next instruction
            // Increment in twos because when we create the opcodes
            // we combine 2 values from memory (whatever values we want to add together for example)
            self.position_in_memory += 2;

            // Extract nibbles from bytes.
            // filter with & bit AND operator.
            // then shift to move the bits to the lowest significant place
            // hex is convenient cause each hex represents 4 bits
            // cast cause otherwise it leaves them as u16 from opcode and we want nibbles.
            // Variable definitions can be found in page 161 table 5.2
            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;

            // You can select multiple nibbles by increasing the width of the filter.
            // we dont need to bit shift them cause they're already in lowest significant place
            // let nnn = opcode & 0x0FFF;
            // let kk = opcode & 0x00FF;

            match (c, x, y, d) {
                (0, 0, 0, 0) => { return; }, // terminate execution when opcode 0x0000 is encountered
                (0x8, _, _, 0x4) => self.add_xy(x, y), // 8 - uses 2 registers, 4 - addition
                _ => todo!("opcode {:04x}", opcode) // add more functionality
            }
        }
    }

    // Add y to x register
    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        // overflowing add returns a tuple, the value and boolean letting us know if overflow was detected
        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        // within CHIP-8, the last register is a 'carry flag'. When set it indiciates
        // that an operation has overflowed the u8 register size.
        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

}


fn main() {
    // Values are held in the registers
    // Instructions on what to do with them are decoded from memory
    let mut cpu = CPU {
        // repeat expressions [x; N], which produces an array with N copies of x
        registers: [0; 16],
        memory: [0; 4096],
        position_in_memory: 0
    };

    // load some data into our registers for processing
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;
    cpu.registers[2] = 10;
    cpu.registers[3] = 10;

    let mem = &mut cpu.memory;

    // load instructions into memory
    mem[0] = 0x80; mem[1] = 0x14; // load opcode 0x8014, which adds register 1 to register 0
    mem[2] = 0x80; mem[3] = 0x24; // load opcode 0x8024, which adds register 2 to register 0
    mem[4] = 0x80; mem[5] = 0x34; // load opcode 0x8034, which adds register 3 to register 0

    // execute main cpu loop
    cpu.run();

    // 5 + 10 + 10 + 10
    assert_eq!(cpu.registers[0], 35);
}

