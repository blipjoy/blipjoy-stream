use bitflags::bitflags;

#[derive(Debug, Default)]
pub struct EaterVm {
    mem: [u8; 16],
    pc: u8,
    a: u8,
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

impl EaterVm {
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

        let inst = self.mem[self.pc as usize];
        let x = inst & 0xf;
        let opcode = inst >> 4;

        self.pc += 1;
        self.pc &= 0xf;

        match opcode {
            0x0 => {
                // NOP
            }
            0x1 => {
                // LDA X
                self.a = self.mem[x as usize];
            }
            0x2 => {
                // ADD X
                let carry = (self.a as u16).wrapping_add(self.mem[x as usize] as u16);

                self.a = carry as u8;
                self.flags = if self.a == 0 {
                    Flags::Z
                } else if carry >= 0x100 {
                    Flags::C
                } else {
                    Flags::CLEAR
                };
            }
            0x3 => {
                // SUB X
                let carry = (self.a as u16).wrapping_sub(self.mem[x as usize] as u16);

                self.a = carry as u8;
                self.flags = if self.a == 0 {
                    Flags::Z
                } else if carry >= 0x100 {
                    Flags::C
                } else {
                    Flags::CLEAR
                };
            }
            0x4 => {
                // STA X
                self.mem[x as usize] = self.a;
            }
            0x5 => {
                // LDI X
                self.a = x;
            }
            0x6 => {
                // JMP X
                self.pc = x;
            }
            0x7 => {
                // JC X
                if self.flags & Flags::C == Flags::C {
                    self.pc = x;
                }
            }
            0x8 => {
                // JZ X
                if self.flags & Flags::Z == Flags::Z {
                    self.pc = x;
                }
            }
            0xe => {
                // OUT
                println!("{}", self.a);
            }
            0xf => {
                // HLT
                self.halt = true;
            }
            _ => {
                panic!("Unknown opcode: {:x?}", opcode);
            }
        }

        self.halt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_nop() {
        let mut vm = EaterVm::new();

        vm.step();
        assert_eq!(vm.pc, 1);
        assert_eq!(vm.a, 0);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(vm.mem, [0; 16]);
    }

    #[test]
    fn test_vm_lda() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x1a; // LDA 10
        vm.mem[10] = 0xa5; // This is the value loaded

        vm.step();
        assert_eq!(vm.pc, 1);
        assert_eq!(vm.a, 0xa5);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(
            vm.mem,
            [0x1a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xa5, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_add_carry() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x2f; // ADD 15
        vm.mem[1] = 0x2f; // ADD 15
        vm.mem[15] = 0xff; // This is the value added

        // Step the interpreter twice to check the flags
        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 0xfe);
        assert_eq!(vm.flags, Flags::C);
        assert_eq!(
            vm.mem,
            [0x2f, 0x2f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff]
        );
    }

    #[test]
    fn test_vm_add_zero() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x2f; // ADD 15
        vm.mem[1] = 0x2f; // ADD 15
        vm.mem[15] = 0; // This is the value added

        // Step the interpreter twice to check the flags
        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 0);
        assert_eq!(vm.flags, Flags::Z);
        assert_eq!(
            vm.mem,
            [0x2f, 0x2f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_add() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x2f; // ADD 15
        vm.mem[1] = 0x2f; // ADD 15
        vm.mem[15] = 1; // This is the value added

        // Step the interpreter twice to check the flags
        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 2);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(
            vm.mem,
            [0x2f, 0x2f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
    }

    #[test]
    fn test_vm_sub_carry() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x3f; // SUB 15
        vm.mem[1] = 0x3f; // SUB 15
        vm.mem[15] = 0xff; // This is the value added

        // Step the interpreter twice to check the flags
        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 2);
        assert_eq!(vm.flags, Flags::C);
        assert_eq!(
            vm.mem,
            [0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff]
        );
    }

    #[test]
    fn test_vm_sub_zero() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x3f; // SUB 15
        vm.mem[1] = 0x3f; // SUB 15
        vm.mem[15] = 0; // This is the value added

        // Step the interpreter twice to check the flags
        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 0);
        assert_eq!(vm.flags, Flags::Z);
        assert_eq!(
            vm.mem,
            [0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_sub() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x3f; // SUB 15
        vm.mem[1] = 0x3f; // SUB 15
        vm.mem[15] = 1; // This is the value added

        // Step the interpreter twice to check the flags
        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 0xfe);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(
            vm.mem,
            [0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
    }

    #[test]
    fn test_vm_sta() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x53; // LDI 3
        vm.mem[1] = 0x4a; // STA 10

        vm.step();
        vm.step();

        assert_eq!(vm.pc, 2);
        assert_eq!(vm.a, 3);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(
            vm.mem,
            [0x53, 0x4a, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_ldi() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x5a; // LDI 10

        vm.step();
        assert_eq!(vm.pc, 1);
        assert_eq!(vm.a, 10);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(vm.mem, [0x5a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_vm_jmp() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x6a; // JMP 10

        vm.step();
        assert_eq!(vm.pc, 10);
        assert_eq!(vm.a, 0);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(vm.mem, [0x6a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_vm_jc() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x7a; // JC 10
        vm.mem[1] = 0x52; // LDI 2
        vm.mem[2] = 0x2f; // ADD 15
        vm.mem[3] = 0x7a; // JC 10
        vm.mem[15] = 0xff;

        for _ in 0..4 {
            vm.step();
        }

        assert_eq!(vm.pc, 10);
        assert_eq!(vm.a, 1);
        assert_eq!(vm.flags, Flags::C);
        assert_eq!(
            vm.mem,
            [0x7a, 0x52, 0x2f, 0x7a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff]
        );
    }

    #[test]
    fn test_vm_jz() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0x8a; // JZ 10
        vm.mem[1] = 0x2f; // ADD 15
        vm.mem[2] = 0x8a; // JZ 10

        for _ in 0..3 {
            vm.step();
        }

        assert_eq!(vm.pc, 10);
        assert_eq!(vm.a, 0);
        assert_eq!(vm.flags, Flags::Z);
        assert_eq!(
            vm.mem,
            [0x8a, 0x2f, 0x8a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_vm_hlt() {
        let mut vm = EaterVm::new();

        vm.mem[0] = 0xf0; // HLT

        for _ in 0..3 {
            vm.step();
        }

        assert_eq!(vm.pc, 1);
        assert_eq!(vm.a, 0);
        assert_eq!(vm.flags, Flags::CLEAR);
        assert_eq!(vm.mem, [0xf0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
