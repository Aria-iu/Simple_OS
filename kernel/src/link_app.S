    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 7
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_6_start
    .quad app_6_end
    .global _app_names
_app_names:
    .string "00hello_world"
    .string "01store_fault"
    .string "02power"
    .string "03priv_inst"
    .string "04priv_csr"
    .string "05get_time"
    .string "06test_datain"
    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/00hello_world.bin"
app_0_end:
            
    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/01store_fault.bin"
app_1_end:
            
    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/02power.bin"
app_2_end:
            
    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/03priv_inst.bin"
app_3_end:
            
    .section .data
    .global app_4_start
    .global app_4_end
    .align 3
app_4_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/04priv_csr.bin"
app_4_end:
            
    .section .data
    .global app_5_start
    .global app_5_end
    .align 3
app_5_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/05get_time.bin"
app_5_end:
            
    .section .data
    .global app_6_start
    .global app_6_end
    .align 3
app_6_start:
    .incbin "../apps/target/riscv64gc-unknown-none-elf/release/06test_datain.bin"
app_6_end:
            
