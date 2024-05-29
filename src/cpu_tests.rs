#[cfg(test)]
mod test {
   use crate::cpu::*;

   #[test]
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
       assert_eq!(cpu.register_a, 0x05);
       assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
       assert!(cpu.status.bits() & 0b1000_0000 == 0);
   }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
    }

    #[test]
   fn test_0xaa_tax_move_a_to_x() {
       let mut cpu = CPU::new();
       cpu.register_a = 10;
       cpu.program_counter = 0x8000;
       cpu.load(vec![0xaa, 0x00]);
       cpu.run();
 
       assert_eq!(cpu.register_x, 10)
   }

   #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
 
       assert_eq!(cpu.register_x, 0xc1)
   }

    #[test]
   fn test_lda_from_memory() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

       assert_eq!(cpu.register_a, 0x55);
   }

   #[test]
   fn test_sta_from_memory() {
       let mut cpu = CPU::new();
       cpu.register_a = 0x55;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x85, 0x10, 0x00]);
       cpu.run();

       assert_eq!(cpu.mem_read(0x10), 0x55);
   }

   #[test]
   fn test_and_from_memory() {
       let mut cpu = CPU::new();
       cpu.register_a = 0x55;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x29, 0x32, 0x00]);
       cpu.run();

       assert_eq!(cpu.register_a, 0x55 & 0x32);
   }

   #[test]
   fn test_adc_no_carry() {
       let mut cpu = CPU::new();
       cpu.register_a = 0x05;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x69, 0x05, 0x00]);
       cpu.run();

       assert_eq!(cpu.register_a, 0x0a);
       assert!(!cpu.status.contains(CpuFlags::CARRY));
   }

   #[test]
   fn test_adc_with_carry() {
       let mut cpu = CPU::new();
       cpu.register_a = 0xa1;
       let val:u8 = 0xa1;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x69, 0xb2, 0x00]);
       cpu.run();

       assert_eq!(cpu.register_a, val.wrapping_add(0xb2));
       assert!(cpu.status.contains(CpuFlags::CARRY));
   }
   #[test]
    fn test_sbc_positive() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x9;
        cpu.program_counter = 0x8000;
        cpu.clear_carry_flag();

        cpu.load(vec![0xe9, 0x04, 0x00]);
        cpu.run();

        println!("{}", cpu.status.bits());
        

        assert_eq!(cpu.register_a, 0x05 - (1 - cpu.status.contains(CpuFlags::CARRY) as u8));
        assert!(!cpu.status.contains(CpuFlags::NEGATIV));
    }

    #[test]
    fn test_sbc_with_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xb2;
        let val:u8 = 0xb2;
        cpu.program_counter = 0x8000;

        cpu.load(vec![0xe9, 0xa1, 0x00]);
        cpu.run();

        assert_eq!(cpu.register_a, val.wrapping_sub(0xa1).wrapping_sub(1));
    }


   #[test]
   fn test_asl_accumulator() {
       let mut cpu = CPU::new();
       cpu.register_a = 0x10;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x0a, 0x00]);
       cpu.run();

       assert_eq!(cpu.register_a, 0x10 << 1);
       assert!(cpu.status.bits() & 0b0000_0001 == 0b00);
   }

   #[test]
   fn test_asl_accumulator_with_carry() {
       let mut cpu = CPU::new();
       cpu.register_a = 0xf5;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x0a, 0x00]);
       cpu.run();

       assert_eq!(cpu.register_a, 0xf5 << 1);
       assert!(cpu.status.bits() & 0b0000_0001 == 0b01);
   }

   #[test]
   fn test_asl_from_memory() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x20);
       

       cpu.load_and_run(vec![0x06, 0x10, 0x00]);

       assert_eq!(cpu.mem_read(0x10), 0x20 << 1);
       assert!(cpu.status.bits() & 0b0000_0001 == 0b00);
   }   

   #[test]
   fn test_asl_from_memory_with_carry() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0xf5);
       cpu.register_a = 0x10;
       cpu.program_counter = 0x8000;
       

       cpu.load(vec![0x06, 0x10, 0x00]);
       cpu.run();

       assert_eq!(cpu.mem_read(0x10), 0xf5 << 1);
       assert!(cpu.status.bits() & 0b0000_0001 == 0b01);
   }  

   #[test]
   fn test_dec_normal() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x10);
       cpu.load_and_run(vec![0xc6, 0x10, 0x00]);
       assert_eq!(cpu.mem_read(0x10), 0x0f);
       assert!(!cpu.status.contains(CpuFlags::NEGATIV));
   }  

   #[test]
   fn test_dec_at_0() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x00);
        cpu.load_and_run(vec![0xc6, 0x10, 0x00]);
        assert_eq!(cpu.mem_read(0x10), 0xff);
        assert!(cpu.status.contains(CpuFlags::NEGATIV));
   }  

   #[test]
   fn test_dex_normal() {
       let mut cpu = CPU::new();
       cpu.register_x = 0x10;
       cpu.program_counter = 0x8000;
       cpu.load(vec![0xca, 0x00]);
       cpu.run();
       assert_eq!(cpu.register_x, 0x0f);
       assert!(!cpu.status.contains(CpuFlags::NEGATIV));
   }  

   #[test]
   fn test_dex_at_0() {
    let mut cpu = CPU::new();
    cpu.register_x = 0x00;
    cpu.program_counter = 0x8000;
    cpu.load(vec![0xca, 0x00]);
    cpu.run();
    assert_eq!(cpu.register_x, 0xff);
    assert!(cpu.status.contains(CpuFlags::NEGATIV));
   }  

   #[test]
   fn test_dey_normal() {
       let mut cpu = CPU::new();
       cpu.register_y = 0x10;
       cpu.program_counter = 0x8000;
       cpu.load(vec![0x88, 0x00]);
       cpu.run();
       assert_eq!(cpu.register_y, 0x0f);
       assert!(!cpu.status.contains(CpuFlags::NEGATIV));
   }  

   #[test]
   fn test_dey_at_0() {
    let mut cpu = CPU::new();
    cpu.register_y = 0x00;
    cpu.program_counter = 0x8000;
    cpu.load(vec![0x88, 0x00]);
    cpu.run();
    assert_eq!(cpu.register_y, 0xff);
    assert!(cpu.status.contains(CpuFlags::NEGATIV));
   }  

   #[test]
   fn test_eor_from_memory_imm() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x32;

    cpu.load(vec![0x49, 0x11, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.register_a, 0x32 ^ 0x11);
   }  

  #[test]
   fn test_eor_from_memory_abs() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x32;
    cpu.mem_write(0x11, 0x55);

    cpu.load(vec![0x4d, 0x11, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.register_a, 0x32 ^ 0x55);
   }  

   #[test]
   fn test_inc_memory() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x11, 0x55);

    cpu.load(vec![0xee, 0x11, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.mem_read(0x11), 0x56);
   }  

   #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.program_counter = 0x8000;
        cpu.load(vec![0xe8, 0xe8, 0x00]);
        cpu.run();

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_iny_overflow() {
        let mut cpu = CPU::new();
        cpu.register_y = 0xff;
        cpu.program_counter = 0x8000;
        cpu.load(vec![0xc8, 0xc8, 0x00]);
        cpu.run();

        assert_eq!(cpu.register_y, 1)
    }

    #[test]
   fn test_0xa9_ldx_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa2, 0x05, 0x00]);
       assert_eq!(cpu.register_x, 0x05);
       assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
       assert!(cpu.status.bits() & 0b1000_0000 == 0);
   }

    #[test]
    fn test_0xa9_ldx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x00, 0x00]);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
    }

    #[test]
   fn test_0xa9_ldy_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa0, 0x05, 0x00]);
       assert_eq!(cpu.register_y, 0x05);
       assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
       assert!(cpu.status.bits() & 0b1000_0000 == 0);
   }

    #[test]
    fn test_0xa9_ldy_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x00, 0x00]);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_lsr_from_memory() {
        let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x20);
       
       cpu.load_and_run(vec![0x4e, 0x10, 0x00]);

       assert_eq!(cpu.mem_read(0x10), 0x20 >> 1);
       assert!(cpu.status.bits() & 0b0000_0001 == 0b00);
    }

    #[test]
    fn test_lsr_acc() {
       let mut cpu = CPU::new();
       cpu.register_a = 0x10;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x4a, 0x00]);
       cpu.run();

       assert_eq!(cpu.register_a, 0x10 >> 1);
       assert!(cpu.status.bits() & 0b0000_0001 == 0b00);
    }

    #[test]
   fn test_ora_from_memory_imm() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x32;

    cpu.load(vec![0x09, 0x11, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.register_a, 0x32 | 0x11);
   }  

  #[test]
   fn test_ora_from_memory_abs() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x32;
    cpu.mem_write(0x11, 0x55);

    cpu.load(vec![0x0d, 0x11, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.register_a, 0x32 | 0x55);
   }  

   #[test]
   fn test_pha() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x32;

    cpu.load(vec![0x48, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.stack_pop(), 0x32);
   }

   #[test]
   fn test_php() {
    let mut cpu = CPU::new();
    cpu.set_carry_flag();

    cpu.load(vec![0x08, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.stack_pop(), 0b00110101);
   }

   #[test]
   fn test_php_no_carry() {
    let mut cpu = CPU::new();

    cpu.load(vec![0x08, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.stack_pop(), 0b00110100);
   }

   #[test]
   fn test_pla() {
    let mut cpu = CPU::new();

    cpu.stack_push(0x32);

    cpu.load(vec![0x68, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.register_a, 0x32);
   }

   #[test]
   fn test_plp() {
    let mut cpu = CPU::new();

    cpu.stack_push(0b10000010);

    cpu.load(vec![0x28, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();

    assert_eq!(cpu.status.bits(), 0b1010_0010);
   }

   #[test]
   fn test_stx_from_memory() {
       let mut cpu = CPU::new();
       cpu.register_x = 0x55;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x8e, 0x10, 0x00]);
       cpu.run();

       assert_eq!(cpu.mem_read(0x10), 0x55);
   }

   #[test]
   fn test_sty_from_memory() {
       let mut cpu = CPU::new();
       cpu.register_y = 0x55;
       cpu.program_counter = 0x8000;

       cpu.load(vec![0x8c, 0x10, 0x00]);
       cpu.run();

       assert_eq!(cpu.mem_read(0x10), 0x55);
   }


}