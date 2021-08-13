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
    LatchPC,         // Memory In + Counter Out
    Fetch(u8),       // RAM Out + Instruction In + Counter Enable
    Execute3(Inst3), // Instruction-specific
    Execute4(Inst4), // Instruction-specific
    Execute5(Inst5), // Instruction-specific
}

impl Default for EaterCycle {
    fn default() -> Self {
        EaterCycle::LatchPC
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Inst3 {
    Nop,
    Lda(u8),
    Add(u8),
    Sub(u8),
    Sta(u8),
    Ldi(u8),
    Jmp(u8),
    Jc(u8),
    Jz(u8),
    Out,
    Hlt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Inst4 {
    Nop,
    Lda(u8),
    Add(u8),
    Sub(u8),
    Sta(u8),
    Ldi,
    Jmp,
    Jc,
    Jz,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Inst5 {
    Nop,
    Lda,
    Add(u8),
    Sub(u8),
    Sta,
    Ldi,
    Jmp,
    Jc,
    Jz,
    Out,
}

impl From<u8> for Inst3 {
    fn from(value: u8) -> Self {
        match value >> 4 {
            0x0 => Inst3::Nop,
            0x1 => Inst3::Lda(value),
            0x2 => Inst3::Add(value),
            0x3 => Inst3::Sub(value),
            0x4 => Inst3::Sta(value),
            0x5 => Inst3::Ldi(value & 0xf),
            0x6 => Inst3::Jmp(value & 0xf),
            0x7 => Inst3::Jc(value & 0xf),
            0x8 => Inst3::Jz(value & 0xf),
            0xe => Inst3::Out,
            0xf => Inst3::Hlt,
            _ => panic!("Unknown opcode: {:x?}", value),
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

    fn step(&mut self) -> bool {
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

                EaterCycle::Execute3(Inst3::from(inst))
            }
            EaterCycle::Execute3(inst) => {
                let inst = match inst {
                    Inst3::Nop => Inst4::Nop,
                    Inst3::Lda(addr) => Inst4::Lda(addr),
                    Inst3::Add(addr) => Inst4::Add(addr),
                    Inst3::Sub(addr) => Inst4::Sub(addr),
                    Inst3::Sta(addr) => Inst4::Sta(addr),
                    Inst3::Ldi(imm) => {
                        self.a = imm;
                        Inst4::Ldi
                    }
                    Inst3::Jmp(pc) => {
                        self.pc = pc;
                        Inst4::Jmp
                    }
                    Inst3::Jc(pc) => {
                        if self.flags & Flags::C == Flags::C {
                            self.pc = pc;
                        }
                        Inst4::Jc
                    }
                    Inst3::Jz(pc) => {
                        if self.flags & Flags::Z == Flags::Z {
                            self.pc = pc;
                        }
                        Inst4::Jz
                    }
                    Inst3::Out => {
                        println!("{}", self.a);
                        Inst4::Out
                    }
                    Inst3::Hlt => {
                        self.halt = true;
                        return self.halt;
                    }
                };

                EaterCycle::Execute4(inst)
            }
            EaterCycle::Execute4(inst) => {
                let inst = match inst {
                    Inst4::Nop => Inst5::Nop,
                    Inst4::Lda(addr) => {
                        let addr = addr & 0xf;
                        self.a = self.mem[addr as usize];
                        Inst5::Lda
                    }
                    Inst4::Add(addr) => {
                        let addr = addr & 0xf;
                        Inst5::Add(self.mem[addr as usize])
                    }
                    Inst4::Sub(addr) => {
                        let addr = addr & 0xf;
                        Inst5::Sub(self.mem[addr as usize])
                    }
                    Inst4::Sta(addr) => {
                        let addr = addr & 0xf;
                        self.mem[addr as usize] = self.a;
                        Inst5::Sta
                    }
                    Inst4::Ldi => Inst5::Ldi,
                    Inst4::Jmp => Inst5::Jmp,
                    Inst4::Jc => Inst5::Jc,
                    Inst4::Jz => Inst5::Jz,
                    Inst4::Out => Inst5::Out,
                };

                EaterCycle::Execute5(inst)
            }
            EaterCycle::Execute5(inst) => {
                match inst {
                    Inst5::Add(b) => {
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
                    Inst5::Sub(b) => {
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

    pub fn run(&mut self) {
        loop {
            if self.step() {
                break;
            }
        }
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Nop));

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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Lda(0x1f)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Lda(0x1f)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Lda));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Add(0x2f)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Add(0x2f)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Add(0x60)));

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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Add(0x2f)));
        assert_eq!(sim.a, 0x60);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Add(0x2f)));
        assert_eq!(sim.a, 0x60);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Add(0x60)));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Add(0x2f)));
        assert_eq!(sim.a, 0xc0);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Add(0x2f)));
        assert_eq!(sim.a, 0xc0);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Add(0x60)));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Sub(0x3f)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Sub(0x3f)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Sub(0x60)));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Sub(0x3f)));
        assert_eq!(sim.a, 0xa0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Sub(0x3f)));
        assert_eq!(sim.a, 0xa0);

        sim.step();
        assert_eq!(sim.pc, 2);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Sub(0x60)));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Sub(0x3f)));
        assert_eq!(sim.a, 0x40);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Sub(0x3f)));
        assert_eq!(sim.a, 0x40);

        sim.step();
        assert_eq!(sim.pc, 3);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Sub(0x60)));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Sta(0x4f)));
        assert_eq!(sim.a, 0x55);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Sta(0x4f)));
        assert_eq!(sim.a, 0x55);
        assert_eq!(sim.mem[15], 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Sta));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Ldi(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Ldi));
        assert_eq!(sim.a, 0xf);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Ldi));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Jmp(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Jmp));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Jmp));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Jc(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Jc));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Jc));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Jc(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Jc));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Jc));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Jz(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Jz));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Jz));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Jz(0xf)));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute4(Inst4::Jz));
        assert_eq!(sim.a, 0);

        sim.step();
        assert_eq!(sim.pc, 0xf);
        assert_eq!(sim.cycle, EaterCycle::Execute5(Inst5::Jz));
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
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Hlt));
        assert!(!sim.halt);

        sim.step();
        assert_eq!(sim.pc, 1);
        assert_eq!(sim.cycle, EaterCycle::Execute3(Inst3::Hlt));
        assert!(sim.halt);
        assert_eq!(sim.a, 0);
        assert_eq!(sim.flags, Flags::CLEAR);
        assert_eq!(sim.mem, [0xf0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
