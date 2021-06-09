
mod assembly_test {
    use opcode::{Opcode, OpcodeDisassembler, Operation::*, OpcodeType::*};

    #[test]
    fn opcode_display() {
        assert_eq!("CLS", format!("{}", Opcode(CLS, NONE)));
        assert_eq!("JP 0x12a", format!("{}", Opcode(JP, NNN(0x12a))));
        assert_eq!("JP 0x0b2", format!("{}", Opcode(JP, NNN(0xb2))));
        assert_eq!("SE V0, 15", format!("{}", Opcode(SE, XNN(0, 15))));
        assert_eq!("AND Va, V3", format!("{}", Opcode(AND, XY(0xa, 0x3))));
        assert_eq!("SHR V2", format!("{}", Opcode(SHR, X(2))));
        assert_eq!("UNDEFINED", format!("{}", Opcode(UNDEFINED, NONE)));
        assert_eq!("LD I, 0x00c", format!("{}", Opcode(LD, I_NNN(0xc))));
        assert_eq!("JP V0, 0xf3d", format!("{}", Opcode(JP, V0_NNN(0xf3d))));
    }

    #[test]
    fn opcode_disassemble() {
        assert_eq!(Opcode(CLS, NONE), OpcodeDisassembler::disassemble(0x00e0));
        assert_eq!(Opcode(RET, NONE), OpcodeDisassembler::disassemble(0x00ee));
        assert_eq!(Opcode(SYS, NNN(0xE1)), OpcodeDisassembler::disassemble(0x00e1));
        assert_eq!(Opcode(SE, XY(2, 0xb)), OpcodeDisassembler::disassemble(0x52b0));
        assert_eq!(Opcode(RND, XNN(6, 0x3a)), OpcodeDisassembler::disassemble(0xc63a));
        assert_eq!(Opcode(LD, DT_X(4)), OpcodeDisassembler::disassemble(0xf415));
        assert_eq!(Opcode(UNDEFINED, NONE), OpcodeDisassembler::disassemble(0x800a));
    }
}
