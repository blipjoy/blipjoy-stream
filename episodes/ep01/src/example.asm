#include "eater_8bit.asm"

lda 14

loop:
add 15
out
jc halt
jmp loop

halt:
hlt

#addr 14
#d8 0
#d8 3
