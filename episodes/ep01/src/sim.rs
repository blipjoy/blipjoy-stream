use bitflags::bitflags;

#[derive(Debug, Default)]
pub struct EaterSim {
    mem: [u8; 16],
    pc: u8,
    a: u8,
    cycle: EaterCycle,
    flags: Flags,
    halt: bool,
}

bitflags! {
    #[derive(Default)]
    struct Flags: u8 {
        const CLEAR = 0;
        const Z = 0b01;
        const C = 0b10;
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EaterCycle {
    LatchPC,        // Memory In + Counter Out
    Fetch(u8),      // RAM Out + Instruction In + Counter Enable
    Execute3(Inst), // Instruction-specific
    Execute4(Inst), // Instruction-specific
    Execute5(Inst), // Instruction-specific
}

impl Default for EaterCycle {
    fn default() -> Self {
        EaterCycle::LatchPC
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Inst {
    Nop3,
    Nop4,
    Nop5,

    Lda3(u8),
    Lda4(u8),
    Lda5,

    Add3(u8),
    Add4(u8),
    Add5(u8),

    Sub3(u8),
    Sub4(u8),
    Sub5(u8),

    Sta3(u8),
    Sta4(u8),
    Sta5,

    Ldi3(u8),
    Ldi4,
    Ldi5,

    Jmp3(u8),
    Jmp4,
    Jmp5,

    Jc3(u8),
    Jc4,
    Jc5,

    Jz3(u8),
    Jz4,
    Jz5,

    Out3,
    Out4,
    Out5,

    Hlt,
}

impl From<u8> for Inst {
    fn from(value: u8) -> Self {
        match value >> 4 {
            0x0 => Inst::Nop3,
            0x1 => Inst::Lda3(value & 0xf),
            0x2 => Inst::Add3(value & 0xf),
            0x3 => Inst::Sub3(value & 0xf),
            0x4 => Inst::Sta3(value & 0xf),
            0x5 => Inst::Ldi3(value & 0xf),
            0x6 => Inst::Jmp3(value & 0xf),
            0x7 => Inst::Jc3(value & 0xf),
            0x8 => Inst::Jz3(value & 0xf),
            0xe => Inst::Out3,
            0xf => Inst::Hlt,
            _ => panic!("Unknown instruction: {:x?}", value),
        }
    }
}

impl EaterSim {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, mem: &[u8]) {
        // TODO: Return Result when mem slice length is not equal to 16.
        self.mem.copy_from_slice(mem);
    }

    pub fn step(&mut self) -> bool {
        if self.halt {
            return self.halt;
        }

        self.cycle = match self.cycle {
            EaterCycle::LatchPC => {
                let pc = self.pc;
                EaterCycle::Fetch(pc)
            }
            EaterCycle::Fetch(pc) => {
                self.pc += 1;
                self.pc &= 0xf;

                let addr = (pc & 0xf) as usize;
                let inst = self.mem[addr];

                EaterCycle::Execute3(Inst::from(inst))
            }
            EaterCycle::Execute3(inst) => {
                let inst = match inst {
                    Inst::Nop3 => Inst::Nop4,
                    Inst::Lda3(addr) => Inst::Lda4(addr),
                    Inst::Add3(addr) => Inst::Add4(addr),
                    Inst::Sub3(addr) => Inst::Sub4(addr),
                    Inst::Sta3(addr) => Inst::Sta4(addr),
                    Inst::Ldi3(imm) => {
                        self.a = imm;
                        Inst::Ldi4
                    }
                    Inst::Jmp3(pc) => {
                        self.pc = pc;
                        Inst::Jmp4
                    }
                    Inst::Jc3(pc) => {
                        if self.flags & Flags::C == Flags::C {
                            self.pc = pc;
                        }
                        Inst::Jc4
                    }
                    Inst::Jz3(pc) => {
                        if self.flags & Flags::Z == Flags::Z {
                            self.pc = pc;
                        }
                        Inst::Jz4
                    }
                    Inst::Out3 => {
                        println!("{}", self.a);
                        Inst::Out4
                    }
                    Inst::Hlt => {
                        self.halt = true;
                        return self.halt;
                    }
                    _ => todo!(),
                };

                EaterCycle::Execute4(inst)
            }
            EaterCycle::Execute4(inst) => {
                let inst = match inst {
                    Inst::Nop4 => Inst::Nop5,
                    Inst::Lda4(addr) => {
                        self.a = self.mem[addr as usize];
                        Inst::Lda5
                    }
                    Inst::Add4(addr) => Inst::Add5(self.mem[addr as usize]),
                    Inst::Sub4(addr) => Inst::Sub5(self.mem[addr as usize]),
                    Inst::Sta4(addr) => {
                        self.mem[addr as usize] = self.a;
                        Inst::Sta5
                    }
                    Inst::Ldi4 => Inst::Ldi5,
                    Inst::Jmp4 => Inst::Jmp5,
                    Inst::Jc4 => Inst::Jc5,
                    Inst::Jz4 => Inst::Jz5,
                    Inst::Out4 => Inst::Out5,
                    _ => todo!(),
                };

                EaterCycle::Execute5(inst)
            }
            EaterCycle::Execute5(inst) => {
                match inst {
                    Inst::Add5(b) => {
                        let carry = (self.a as u16).wrapping_add(b as u16);

                        self.a = carry as u8;
                        self.flags = if self.a == 0 {
                            Flags::Z
                        } else if carry >= 0x100 {
                            Flags::C
                        } else {
                            Flags::CLEAR
                        };
                    }
                    Inst::Sub5(b) => {
                        let carry = (self.a as u16).wrapping_sub(b as u16);

                        self.a = carry as u8;
                        self.flags = if self.a == 0 {
                            Flags::Z
                        } else if carry >= 0x100 {
                            Flags::C
                        } else {
                            Flags::CLEAR
                        };
                    }
                    _ => (),
                };

                EaterCycle::LatchPC
            }
        };

        self.halt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_nop() {
        let mut sim = EaterSim::new();

        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Nop3));

        for _ in 0..3 {
            sim.step();
        }

        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(sim.mem, [0; 16]);
    }

    #[test]
    fn test_vm_lda() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x1f; // LDA 15
        sim.mem[15] = 0x55; // Value to load

        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Lda3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Lda4(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Lda5));
        assert_eq!(sim.a, 0x55);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0x55);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(
            sim.mem,
            [0x1f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x55]
        );
    }

    #[test]
    fn test_vm_add() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x2f; // ADD 15
        sim.mem[1] = 0x2f; // ADD 15
        sim.mem[2] = 0x2f; // ADD 15
        sim.mem[15] = 0x60; // Value to load

        // First instruction
        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Add3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Add4(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Add5(0x60)));

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0x60);

        // Second instruction
        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Fetch(1));
        assert_eq!(sim.a, 0x60);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Add3(0xf)));
        assert_eq!(sim.a, 0x60);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Add4(0xf)));
        assert_eq!(sim.a, 0x60);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Add5(0x60)));
        assert_eq!(sim.a, 0x60);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0xc0);

        // Third instruction
        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Fetch(2));
        assert_eq!(sim.a, 0xc0);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Add3(0xf)));
        assert_eq!(sim.a, 0xc0);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Add4(0xf)));
        assert_eq!(sim.a, 0xc0);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Add5(0x60)));
        assert_eq!(sim.a, 0xc0);
        assert_eq!(sim.flags, Flags::CLEAR);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0x20);
        assert_eq!(sim.flags, Flags::C);
        assert_eq!(
            sim.mem,
            [0x2f, 0x2f, 0x2f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x60]
        );
    }

    #[test]
    fn test_vm_sub() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x3f; // SUB 15
        sim.mem[1] = 0x3f; // SUB 15
        sim.mem[2] = 0x3f; // SUB 15
        sim.mem[15] = 0x60; // Value to load

        // First instruction
        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Sub3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Sub4(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Sub5(0x60)));
        assert_eq!(sim.flags, Flags::CLEAR);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0xa0);
        assert_eq!(sim.flags, Flags::C);

        // Second instruction
        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Fetch(1));
        assert_eq!(sim.a, 0xa0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Sub3(0xf)));
        assert_eq!(sim.a, 0xa0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Sub4(0xf)));
        assert_eq!(sim.a, 0xa0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Sub5(0x60)));
        assert_eq!(sim.a, 0xa0);
        assert_eq!(sim.flags, Flags::C);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0x40);
        assert_eq!(sim.flags, Flags::CLEAR);

        // Third instruction
        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Fetch(2));
        assert_eq!(sim.a, 0x40);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Sub3(0xf)));
        assert_eq!(sim.a, 0x40);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Sub4(0xf)));
        assert_eq!(sim.a, 0x40);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Sub5(0x60)));
        assert_eq!(sim.a, 0x40);
        assert_eq!(sim.flags, Flags::CLEAR);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0xe0);
        assert_eq!(sim.flags, Flags::C);
        assert_eq!(
            sim.mem,
            [0x3f, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x60]
        );
    }

    #[test]
    fn test_vm_sta() {
        let mut sim = EaterSim::new();

        sim.a = 0x55;
        sim.mem[0] = 0x4f; // STA 15

        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0x55);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Sta3(0xf)));
        assert_eq!(sim.a, 0x55);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Sta4(0xf)));
        assert_eq!(sim.a, 0x55);
        assert_eq!(sim.mem[15], 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Sta5));
        assert_eq!(sim.a, 0x55);
        assert_eq!(sim.mem[15], 0x55);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0x55);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(
            sim.mem,
            [0x4f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x55]
        );
    }

    #[test]
    fn test_vm_ldi() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x5f; // LDI 15

        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Ldi3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Ldi4));
        assert_eq!(sim.a, 0xf);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Ldi5));
        assert_eq!(sim.a, 0xf);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0xf);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(sim.mem, [0x5f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_vm_jmp() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x6f; // JMP 15

        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Jmp3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Jmp4));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Jmp5));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(sim.mem, [0x6f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_vm_jc() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x7f; // JC 15
        sim.mem[1] = 0x7f; // JC 15

        // First instruction
        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Jc3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Jc4));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Jc5));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::CLEAR);

        // HAXX
        sim.flags = Flags::C;

        // Second instruction
        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Fetch(1));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Jc3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Jc4));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Jc5));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::C);
        assert_eq!(
            sim.mem,
            [0x7f, 0x7f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_jz() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0x8f; // JZ 15
        sim.mem[1] = 0x8f; // JZ 15

        // First instruction
        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Jz3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Jz4));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Jz5));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::CLEAR);

        // HAXX
        sim.flags = Flags::Z;

        // Second instruction
        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Fetch(1));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Jz3(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst::Jz4));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst::Jz5));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::LatchPC);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::Z);
        assert_eq!(
            sim.mem,
            [0x8f, 0x8f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_hlt() {
        let mut sim = EaterSim::new();

        sim.mem[0] = 0xf0; // HLT

        sim.step();
        assert_eq!(sim.pc, 0);
        assert_eq!(sim.cycle, EaterCycle::Fetch(0));

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Hlt));
        assert!(!sim.halt);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst::Hlt));
        assert!(sim.halt);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(sim.mem, [0xf0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
