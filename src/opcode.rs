#[allow(non_camel_case_types)]
pub enum OpcodeType {
    SPECIAL,            // **** -> exact match
    NNN(u16),           // *NNN -> address nnn
    XNN(u16, u16),      // *XNN -> Vx, value nn
    XY(u16, u16),       // *XY* -> Vx, Vy
    XYN(u16, u16, u16), // *XYN -> Vx, Vy, value n
    X(u16),             // *X** -> Vx
    I_X(u16),           // *X** -> I, Vx
    V0_NNN(u16),        // *X** -> V0, nnn
    X_K(u16),           // *X** -> Vx, K
    DT_X(u16),          // *X** -> DT, Vx
    X_DT(u16),          // *X** -> Vx, DT
    F_X(u16),           // *X** -> F, Vx
    B_X(u16),           // *X** -> B, Vx
    ST_X(u16),          // *X** -> ST, Vx
    RI_X(u16),          // *X** -> [I], Vx
    X_RI(u16)           // *X** -> Vx, [I]
}
use self::OpcodeType::*;

pub enum Opcode {
    SYS(OpcodeType),
    CLS(OpcodeType),
    RET(OpcodeType),
    JP(OpcodeType),
    CALL(OpcodeType),
    SE(OpcodeType),
    SNE(OpcodeType),
    LD(OpcodeType),
    ADD(OpcodeType),
    OR(OpcodeType),
    AND(OpcodeType),
    XOR(OpcodeType),
    SUB(OpcodeType),
    SHR(OpcodeType),
    SUBN(OpcodeType),
    SHL(OpcodeType),
    RND(OpcodeType),
    DRW(OpcodeType),
    SKP(OpcodeType),
    SKNP(OpcodeType),
    UNDEFINED
}
use self::Opcode::*;

trait Disassembler {
    type Instruction;
    type Opcode;
    fn disassemble(Self::Instruction) -> Opcode;
}

struct OpcodeDisassembler {}

impl Disassembler for OpcodeDisassembler {
    type Instruction = u16;
    type Opcode = Opcode;

    fn disassemble(opcode: u16) -> Opcode {
        let u = opcode >> 12;       //u___
        let nnn = opcode & 0xFFF;   //_nnn
        let nn = opcode & 0xFF;     //__nn
        let n = opcode & 0xF;       //___n
        let x = nnn >> 8;           //_x__
        let y = nn >> 4;            //__y_
        
        match opcode {
            0x00E0 => CLS(SPECIAL),         // Clear the screen
            0x00EE => RET(SPECIAL),         // Return from subroutine
            _ => match u {
                0x0 => SYS(NNN(nnn)),       // Calls RCA 1802 program
                0x1 => JP(NNN(nnn)),        // Jump NNN
                0x2 => CALL(NNN(nnn)),      // Call subroutine NNN
                0x3 => SE(XNN(x, nn)),      // Skip if Vx == NN
                0x4 => SNE(XNN(x, nn)),     // Skip if Vx != NN
                0x5 => match n {
                    0 => SE(XY(x, y)),      // Skip if Vx == Vy
                    _ => UNDEFINED
                },
                0x6 => LD(XNN(x, nn)),      // Vx = NN
                0x7 => ADD(XNN(x, nn)),     // Vx += NN
                0x8 => match n {
                    0x0 => LD(XY(x, y)),    // Vx = Vy
                    0x1 => OR(XY(x, y)),    // Vx = Vx | Vy
                    0x2 => AND(XY(x, y)),   // Vx = Vx & Vy
                    0x3 => XOR(XY(x, y)),   // Vx = Vx ^ Vy
                    0x4 => ADD(XY(x, y)),   // Vx += Vy
                    0x5 => SUB(XY(x, y)),   // Vx -= Vy
                    0x6 => SHR(X(x)),       // Vx >> 1
                    0x7 => SUBN(XY(x, y)),  // Vx = Vy - Vx
                    0xE => SHL(X(x)),       // Vx << 1
                    _ => UNDEFINED
                },
                0x9 => match n {
                    0 => SNE(XY(x, y)),     // Skip if Vx!=Vy
                    _ => UNDEFINED
                },
                0xA => LD(NNN(nnn)),        // I = NNN
                0xB => JP(V0_NNN(nnn)),     // PC = V0 + NNN
                0xC => RND(XNN(x, nn)),     // Vx = rand() & NN
                0xD => DRW(XYN(x, y, n)),   // Display sprite I at (Vx, Vy)
                0xE => match nn {
                    0x9E => SKP(X(x)),      // Skip if key Vx is pressed
                    0xA1 => SKNP(X(x)),     // Skip if key Vx is not pressed
                    _ => UNDEFINED
                },
                0xF => match nn {
                    0x07 => LD(X_DT(x)),    // Vx = delay timer value
                    0x0A => LD(X_K(x)),     // Wait for key, then store in Vx
                    0x15 => LD(DT_X(x)),    // Delay timer = Vx
                    0x18 => LD(ST_X(x)),    // Sound timer = Vx
                    0x1E => ADD(I_X(x)),    // I += Vx
                    0x29 => LD(F_X(x)),     // I = sprite_address[Vx]
                    0x33 => LD(B_X(x)),     // Vx to decimal in [I, I+1, I+2]
                    0x55 => LD(RI_X(x)),    // [I..I+x] = [V0..Vx]
                    0x65 => LD(X_RI(x)),    // [V0..Vx] = [I..I+x]
                    _ => UNDEFINED
                },
                _ => UNDEFINED
            }
        }
    }
}