; Eater 8-bit CPU

#bits 8

#ruledef eater_8bit {
    nop => 0x00
    lda {mem: u4} => 0x1 @ mem
    add {mem: u4} => 0x2 @ mem
    sub {mem: u4} => 0x3 @ mem
    sta {mem: u4} => 0x4 @ mem
    ldi {mem: u4} => 0x5 @ mem
    jmp {mem: u4} => 0x6 @ mem
    jc {mem: u4} => 0x7 @ mem
    jz {mem: u4} => 0x8 @ mem
    out => 0xe0
    hlt => 0xf0
}
