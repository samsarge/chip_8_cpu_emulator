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

struct CPU {
    current_operation: u16, // All CHIP-8 opcodes are U16 values, defined by who makes the architecture
    registers: [u8; 2] // just 2 registers now
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        self.current_operation // TODO: Eventually read from memory
    }

    /// Main CPU loop
    /// 1. Reads the opcode
    /// 2. Decodes instructions
    /// 3. Matches decoded instructions to known opcodes
    /// 4. dispatches execution of the operation to a specific function
    fn run(&mut self) {
        // loop { // TODO: implement main loop
            let opcode = self.read_opcode();

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
            let nnn = opcode & 0x0FFF;
            let kk = opcode & 0x00FF;

            match (c, x, y, d) {
                (0x8, _, _, 0x4) => self.add_xy(x, y), // 8 - uses 2 registers, 4 - addition
                _ => todo!("opcode {:04x}", opcode) // add more functionality
            }
        // }
    }

    // Add y to x register
    fn add_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] += self.registers[y as usize];
    }

}


fn main() {
    let mut cpu = CPU {
        current_operation: 0, // init with no-op (do nothing)
        registers: [0; 2] // A repeat expression [x; N], which produces an array with N copies of x
    };

    println!("Initialised with current_operation: {:?} | registers: {:?}", cpu.current_operation, cpu.registers);

    
    cpu.current_operation = 0x8014;
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    cpu.run();

    assert_eq!(cpu.registers[0], 15);
}

