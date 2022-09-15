// Ref: rust in action

// CHIP-8 Emulator.

// Decoding CHIP-8 opcodes.
// they're u16 values made up of 4 nibbles (half a byte / 4 bits)
// rust has no 4 bit type so we just wrap them all together in a 16bit type and then awkwardly
// filter out the bits we dont care about to access each individual nibble

// CHIP-8 breaks opcodes down into 2 bytes, high byte, low byte, then nibbles, high nibble, low nibble
// (layout of the bytes is called endianness, cpu manufacturers decide this)
// 0xAB12
// high byte = AB
  // high nibble A
  // low nibble B
// low byte = 12
  // high nibble 1
  // low nibble 2

// Note that control flow in a CPU is done by comparing values in a register
// then modifying position_in_memory, depending on the outcome. There are no while
// or for loops in the CPU, thats the job of the programming languages compiler.
use core::panic;

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
    memory: [u8; 0x1000],

    // ~ The stack ~ specialised memory for CALL and RETURN opcodes
    stack: [u16; 16], // stacks maximum height is 16m after 16 nested function calls we say its a stack overflow
    stack_pointer: usize // giving the stack_pointer usize makes it easier to index values cause rust
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

            // To support functions
            let nnn = opcode & 0x0FFF;
            let kk = opcode & 0x00FF;

            // 

            match (c, x, y, d) {
                (0, 0, 0, 0) => { return; }, // terminate execution when opcode 0x0000 is encountered
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(nnn),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!("opcode {:04x}", opcode) // add more functionality
            }
        }
    }

    // ADD_XY: Add y to x register
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

    // CALL: opcode 0x2nnn sets position_in_memory to nnn, the address of the function
    // Each CALL opcode adds an address to the stack by incrementing the stack pointer
    // and writing nnn to that position in the stack.

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;   
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow");
        }

        // add current position in memory to stack
        // memory address is two bytes higher than calling location as it is incremented within the body of run()
        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;

        // modify position in memory to affect jumping to that address
        self.position_in_memory = addr as usize;
    }

    // RETURN: opcode 0x00EE sets position_in_memory to the memory address of the previous CALL opcode
    // Each RETURN opcode removes the top address by decrementing the stack pointer.
    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        // jump to position in memory where an earlier call was made
        let call_addr = self.stack[self.stack_pointer];
        self.position_in_memory = call_addr as usize;
    }
}

fn main() {
    // Values are held in the registers
    // Instructions on what to do with them are decoded from memory
    let mut cpu = CPU {
        // repeat expressions [x; N], which produces an array with N copies of x
        registers: [0; 16],
        memory: [0; 4096],
        position_in_memory: 0,
        stack: [0; 16],
        stack_pointer: 0
    };

    // Use our CPU to calculate: 5 + (10 * 2) + (10 * 2) = 45

    // load some data into our registers for processing
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    let mem = &mut cpu.memory;

    // set opcode to 0x2100: CALL the function at 0x100
    mem[0x000] = 0x21; mem[0x001] = 0x00;
    // set opcode to 0x2100: CALL the function at 0x100
    mem[0x002] = 0x21; mem[0x003] = 0x00;
    // sets opcode to 0x0000: HALT (not really needed as cpu.memory is initialized with null bytes)
    mem[0x004] = 0x00; mem[0x005] = 0x00;


    // sets opcode to 0x8014: ADD register 1s value to register 0
    mem[0x100] = 0x80; mem[0x101] = 0x14;
    // sets opcode to 0x8014: ADD register 1s value to register 0
    mem[0x102] = 0x80; mem[0x103] = 0x14;
    // sets opcode to 0x00EE: RETURN
    mem[0x104] = 0x00; mem[0x105] = 0xEE;

    // execute main cpu loop
    cpu.run();

    assert_eq!(cpu.registers[0], 45);

}

