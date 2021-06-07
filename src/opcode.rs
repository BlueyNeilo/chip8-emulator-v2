use std::fmt::{self, Debug, Display, Formatter};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum OpcodeType {
    NONE,               // **** -> exact match
    NNN(u16),           // *NNN -> address nnn
    XNN(u16, u16),      // *XNN -> Vx, value nn
    XY(u16, u16),       // *XY* -> Vx, Vy
    XYN(u16, u16, u16), // *XYN -> Vx, Vy, value n
    X(u16),             // *X** -> Vx
    I_X(u16),           // *X** -> I, Vx
    I_NNN(u16),         // *NNN -> I, nnn
    V0_NNN(u16),        // *X** -> V0, nnn
    X_K(u16),           // *X** -> Vx, K
    DT_X(u16),          // *X** -> DT, Vx
    X_DT(u16),          // *X** -> Vx, DT
    F_X(u16),           // *X** -> F, Vx
    B_X(u16),           // *X** -> B, Vx
    ST_X(u16),          // *X** -> ST, Vx
    RI_X(u16),          // *X** -> [I], Vx
    X_RI(u16),          // *X** -> Vx, [I]
}

impl Display for OpcodeType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let result = match *self {
            NONE =>         String::from(""),
            NNN(nnn) =>     format!("{}", nnn),
            XNN(x, nn) =>   format!("V{}, {}", x, nn),
            XY(x, y) =>     format!("V{}, V{}", x, y),
            XYN(x, y, n) => format!("V{}, V{}, {}", x, y, n),
            X(x) =>         format!("V{}", x),
            I_X(x) =>       format!("I, V{}", x),
            I_NNN(nnn) =>   format!("I, {}", nnn),
            V0_NNN(nnn) =>  format!("V0, {}", nnn),
            X_K(x) =>       format!("V{}, K", x),
            DT_X(x) =>      format!("DT, V{}", x),
            X_DT(x) =>      format!("V{}, DT", x),
            F_X(x) =>       format!("F, V{}", x),
            B_X(x) =>       format!("B, V{}", x),
            ST_X(x) =>      format!("ST, V{}", x),
            RI_X(x) =>      format!("[I], V{}", x),
            X_RI(x) =>      format!("V{}, [I]", x),
        };

        write!(f, "{}", result)
    }
}

use self::OpcodeType::*;

pub struct Opcode(pub Operation, pub OpcodeType);

pub enum Operation {
    SYS,
    CLS,
    RET,
    JP,
    CALL,
    SE,
    SNE,
    LD,
    ADD,
    OR,
    AND,
    XOR,
    SUB,
    SHR,
    SUBN,
    SHL,
    RND,
    DRW,
    SKP,
    SKNP,
    UNDEFINED
}
use self::Operation::*;

trait Disassembler {
    type Instruction;
    type Opcode;
    fn disassemble(Self::Instruction) -> Opcode;
}

struct OpcodeDisassembler;

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
            0x00E0 => Opcode(CLS, NONE),            // Clear the screen
            0x00EE => Opcode(RET, NONE),            // Return from subroutine
            _ => match u {
                0x0 => Opcode(SYS, NNN(nnn)),       // Calls RCA 1802 program
                0x1 => Opcode(JP, NNN(nnn)),        // Jump NNN
                0x2 => Opcode(CALL, NNN(nnn)),      // Call subroutine NNN
                0x3 => Opcode(SE, XNN(x, nn)),      // Skip if Vx == NN
                0x4 => Opcode(SNE, XNN(x, nn)),     // Skip if Vx != NN
                0x5 => match n {
                    0 => Opcode(SE, XY(x, y)),      // Skip if Vx == Vy
                    _ => Opcode(UNDEFINED, NONE)
                },
                0x6 => Opcode(LD, XNN(x, nn)),      // Vx = NN
                0x7 => Opcode(ADD, XNN(x, nn)),     // Vx += NN
                0x8 => match n {
                    0x0 => Opcode(LD, XY(x, y)),    // Vx = Vy
                    0x1 => Opcode(OR, XY(x, y)),    // Vx = Vx | Vy
                    0x2 => Opcode(AND, XY(x, y)),   // Vx = Vx & Vy
                    0x3 => Opcode(XOR, XY(x, y)),   // Vx = Vx ^ Vy
                    0x4 => Opcode(ADD, XY(x, y)),   // Vx += Vy
                    0x5 => Opcode(SUB, XY(x, y)),   // Vx -= Vy
                    0x6 => Opcode(SHR, X(x)),       // Vx >> 1
                    0x7 => Opcode(SUBN, XY(x, y)),  // Vx = Vy - Vx
                    0xE => Opcode(SHL, X(x)),       // Vx << 1
                    _ => Opcode(UNDEFINED, NONE)
                },
                0x9 => match n {
                    0 => Opcode(SNE, XY(x, y)),     // Skip if Vx!=Vy
                    _ => Opcode(UNDEFINED, NONE)
                },
                0xA => Opcode(LD, I_NNN(nnn)),      // I = NNN
                0xB => Opcode(JP, V0_NNN(nnn)),     // PC = V0 + NNN
                0xC => Opcode(RND, XNN(x, nn)),     // Vx = rand() & NN
                0xD => Opcode(DRW, XYN(x, y, n)),   // Display sprite I at (Vx, Vy)
                0xE => match nn {
                    0x9E => Opcode(SKP, X(x)),      // Skip if key Vx is pressed
                    0xA1 => Opcode(SKNP, X(x)),     // Skip if key Vx is not pressed
                    _ => Opcode(UNDEFINED, NONE)
                },
                0xF => match nn {
                    0x07 => Opcode(LD, X_DT(x)),    // Vx = delay timer value
                    0x0A => Opcode(LD, X_K(x)),     // Wait for key, then store in Vx
                    0x15 => Opcode(LD, DT_X(x)),    // Delay timer = Vx
                    0x18 => Opcode(LD, ST_X(x)),    // Sound timer = Vx
                    0x1E => Opcode(ADD, I_X(x)),    // I += Vx
                    0x29 => Opcode(LD, F_X(x)),     // I = sprite_address[Vx]
                    0x33 => Opcode(LD, B_X(x)),     // Vx to decimal in [I, I+1, I+2]
                    0x55 => Opcode(LD, RI_X(x)),    // [I..I+x] = [V0..Vx]
                    0x65 => Opcode(LD, X_RI(x)),    // [V0..Vx] = [I..I+x]
                    _ => Opcode(UNDEFINED, NONE)
                },
                _ => Opcode(UNDEFINED, NONE)
            }
        }
    }
}