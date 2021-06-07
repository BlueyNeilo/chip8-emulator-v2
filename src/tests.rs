
mod assembly_test {
    use opcode::{Opcode,Operation::*,OpcodeType::*};

    #[test]
    fn opcode_display() {
        assert_eq!("CLS", format!("{}", Opcode(CLS, NONE)))
    }
}