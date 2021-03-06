use std::fmt;
use std::fmt::Display;

use log::error;

use super::alu::hi_lo;
use super::cpu::{ConditionCode, Operand, Register16, Register8};
use super::mem::Address;

mod arith;
mod bits;
mod control;
mod load;
mod logic;

pub use self::arith::Arith;
pub use self::bits::Bits;
pub use self::control::Control;
pub use self::load::Load;
pub use self::logic::Logic;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    EnableInterrupts,
    DisableInterrupts,
    Halt,
    SetCarry,
    ClearCarry,
    Stop,
    Compare(Operand),
    Arith(Arith),
    Bits(Bits),
    Control(Control),
    Load(Load),
    Logic(Logic),
}

impl Instruction {
    pub fn cycles(self, branch_taken: bool) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Instruction::Nop => 4,
            Instruction::EnableInterrupts => 4,
            Instruction::DisableInterrupts => 4,
            Instruction::Halt => 4,
            Instruction::Stop => 4,
            Instruction::SetCarry | Instruction::ClearCarry => 4,
            Instruction::Compare(Operand::Immediate(_)) => 8,
            Instruction::Compare(Operand::IndirectRegister(_)) => 8,
            Instruction::Compare(Operand::Register(_)) => 4,

            Instruction::Arith(a) => a.cycles(),
            Instruction::Bits(b) => b.cycles(),
            Instruction::Load(l) => l.cycles(),
            Instruction::Control(c) => c.cycles(branch_taken),
            Instruction::Logic(l) => l.cycles(),

            Instruction::Compare(_) => unimplemented!(),
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Result<(Instruction, u8), ()> {
        match bytes[0] {
            0 => Ok((Instruction::Nop, 1)),

            0xFB => Ok((Instruction::EnableInterrupts, 1)),
            0xF3 => Ok((Instruction::DisableInterrupts, 1)),

            0x10 => match bytes[1] {
                0x00 => Ok((Instruction::Stop, 2)),
                _ => {
                    error!(
                        "Unknown instruction {:#X} {:#X} {:#X}",
                        bytes[0], bytes[1], bytes[2]
                    );
                    Err(())
                }
            },
            0x76 => Ok((Instruction::Halt, 1)),

            0x37 => Ok((Instruction::SetCarry, 1)),
            0x3F => Ok((Instruction::ClearCarry, 1)),

            0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => Ok((
                Instruction::Arith(Arith::Increment(Operand::from_bits(bytes[0], 3))),
                1,
            )),

            0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => Ok((
                Instruction::Arith(Arith::Decrement(Operand::from_bits(bytes[0], 3))),
                1,
            )),

            0x08 => Ok((
                Instruction::Load(Load::LoadMemoryFromSP(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),

            0x0B => Ok((
                Instruction::Arith(Arith::DecrementRegister16(Register16::BC)),
                1,
            )),
            0x1B => Ok((
                Instruction::Arith(Arith::DecrementRegister16(Register16::DE)),
                1,
            )),
            0x2B => Ok((
                Instruction::Arith(Arith::DecrementRegister16(Register16::HL)),
                1,
            )),
            0x3B => Ok((
                Instruction::Arith(Arith::DecrementRegister16(Register16::SP)),
                1,
            )),

            0x03 => Ok((
                Instruction::Arith(Arith::IncrementRegister16(Register16::BC)),
                1,
            )),
            0x13 => Ok((
                Instruction::Arith(Arith::IncrementRegister16(Register16::DE)),
                1,
            )),
            0x23 => Ok((
                Instruction::Arith(Arith::IncrementRegister16(Register16::HL)),
                1,
            )),
            0x33 => Ok((
                Instruction::Arith(Arith::IncrementRegister16(Register16::SP)),
                1,
            )),

            0xE8 => Ok((Instruction::Arith(Arith::AddSP(bytes[1] as i8)), 2)),

            0x09 => Ok((
                Instruction::Arith(Arith::AddRegisterRegister16(Register16::HL, Register16::BC)),
                1,
            )),
            0x19 => Ok((
                Instruction::Arith(Arith::AddRegisterRegister16(Register16::HL, Register16::DE)),
                1,
            )),
            0x29 => Ok((
                Instruction::Arith(Arith::AddRegisterRegister16(Register16::HL, Register16::HL)),
                1,
            )),
            0x39 => Ok((
                Instruction::Arith(Arith::AddRegisterRegister16(Register16::HL, Register16::SP)),
                1,
            )),

            0xE9 => Ok((Instruction::Control(Control::JumpIndirect), 1)),
            0xC3 => Ok((
                Instruction::Control(Control::Jump(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xC2 | 0xD2 | 0xCA | 0xDA => Ok((
                Instruction::Control(Control::JumpConditional(
                    Address(hi_lo(bytes[2], bytes[1])),
                    ConditionCode::from_bits(bytes[0]),
                )),
                3,
            )),

            0xC7 => Ok((Instruction::Control(Control::Reset(Address(0x0000))), 1)),
            0xD7 => Ok((Instruction::Control(Control::Reset(Address(0x0010))), 1)),
            0xE7 => Ok((Instruction::Control(Control::Reset(Address(0x0020))), 1)),
            0xF7 => Ok((Instruction::Control(Control::Reset(Address(0x0030))), 1)),

            0xCF => Ok((Instruction::Control(Control::Reset(Address(0x0008))), 1)),
            0xDF => Ok((Instruction::Control(Control::Reset(Address(0x0018))), 1)),
            0xEF => Ok((Instruction::Control(Control::Reset(Address(0x0028))), 1)),
            0xFF => Ok((Instruction::Control(Control::Reset(Address(0x0038))), 1)),

            0xCD => Ok((
                Instruction::Control(Control::Call(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xC4 | 0xD4 | 0xCC | 0xDC => Ok((
                Instruction::Control(Control::CallConditional(
                    Address(hi_lo(bytes[2], bytes[1])),
                    ConditionCode::from_bits(bytes[0]),
                )),
                3,
            )),

            0xF0 => Ok((
                Instruction::Load(Load::Load(
                    Operand::Register(Register8::A),
                    Operand::IndirectAddress(Address(0xFF00) + Address(u16::from(bytes[1]))),
                )),
                2,
            )),
            0xE0 => Ok((
                Instruction::Load(Load::Load(
                    Operand::IndirectAddress(Address(0xFF00) + Address(u16::from(bytes[1]))),
                    Operand::Register(Register8::A),
                )),
                2,
            )),
            0x01 => Ok((
                Instruction::Load(Load::LoadRegisterImmediate16(
                    Register16::BC,
                    hi_lo(bytes[2], bytes[1]),
                )),
                3,
            )),
            0x11 => Ok((
                Instruction::Load(Load::LoadRegisterImmediate16(
                    Register16::DE,
                    hi_lo(bytes[2], bytes[1]),
                )),
                3,
            )),
            0x21 => Ok((
                Instruction::Load(Load::LoadRegisterImmediate16(
                    Register16::HL,
                    hi_lo(bytes[2], bytes[1]),
                )),
                3,
            )),
            0x31 => Ok((
                Instruction::Load(Load::LoadRegisterImmediate16(
                    Register16::SP,
                    hi_lo(bytes[2], bytes[1]),
                )),
                3,
            )),

            0x2F => Ok((Instruction::Bits(Bits::Complement), 1)),

            0x27 => Ok((Instruction::Arith(Arith::DecimalAdjustAccumulator), 1)),

            0xEA => Ok((
                Instruction::Load(Load::LoadMemoryFromA(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xFA => Ok((
                Instruction::Load(Load::LoadAFromMemory(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),

            0xE2 => Ok((Instruction::Load(Load::LoadIndirectHiFromA), 1)),
            0xF2 => Ok((Instruction::Load(Load::LoadAFromIndirectHi), 1)),

            0xF8 => Ok((Instruction::Load(Load::LoadHLFromSP(bytes[1] as i8)), 2)),
            0xF9 => Ok((Instruction::Load(Load::LoadSPFromHL), 1)),

            0x02 => Ok((
                Instruction::Load(Load::LoadIndirectRegisterFromA(Register16::BC)),
                1,
            )),
            0x12 => Ok((
                Instruction::Load(Load::LoadIndirectRegisterFromA(Register16::DE)),
                1,
            )),

            0x0A => Ok((
                Instruction::Load(Load::LoadAFromIndirectRegister(Register16::BC)),
                1,
            )),
            0x1A => Ok((
                Instruction::Load(Load::LoadAFromIndirectRegister(Register16::DE)),
                1,
            )),

            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => Ok((
                Instruction::Load(Load::Load(
                    Operand::from_bits(bytes[0], 3),
                    Operand::Immediate(bytes[1]),
                )),
                2,
            )),

            0x40..=0x75 | 0x77..=0x7F => Ok((
                Instruction::Load(Load::Load(
                    Operand::from_bits(bytes[0], 3),
                    Operand::from_bits(bytes[0], 0),
                )),
                1,
            )),

            0x22 => Ok((Instruction::Load(Load::LoadIndirectFromA(1)), 1)),
            0x32 => Ok((Instruction::Load(Load::LoadIndirectFromA(-1)), 1)),

            0x2A => Ok((Instruction::Load(Load::LoadAFromIndirect(1)), 1)),
            0x3A => Ok((Instruction::Load(Load::LoadAFromIndirect(-1)), 1)),

            0xC5 => Ok((Instruction::Load(Load::Push(Register16::BC)), 1)),
            0xD5 => Ok((Instruction::Load(Load::Push(Register16::DE)), 1)),
            0xE5 => Ok((Instruction::Load(Load::Push(Register16::HL)), 1)),
            0xF5 => Ok((Instruction::Load(Load::Push(Register16::AF)), 1)),

            0xC1 => Ok((Instruction::Load(Load::Pop(Register16::BC)), 1)),
            0xD1 => Ok((Instruction::Load(Load::Pop(Register16::DE)), 1)),
            0xE1 => Ok((Instruction::Load(Load::Pop(Register16::HL)), 1)),
            0xF1 => Ok((Instruction::Load(Load::Pop(Register16::AF)), 1)),

            0xFE => Ok((Instruction::Compare(Operand::Immediate(bytes[1])), 2)),

            0xB8..=0xBF => Ok((Instruction::Compare(Operand::from_bits(bytes[0], 0)), 1)),

            0x20 | 0x30 | 0x28 | 0x38 => Ok((
                Instruction::Control(Control::JumpRelativeConditional(
                    bytes[1] as i8,
                    ConditionCode::from_bits(bytes[0]),
                )),
                2,
            )),
            0x18 => Ok((
                Instruction::Control(Control::JumpRelative(bytes[1] as i8)),
                2,
            )),

            0x80..=0x87 => Ok((
                Instruction::Arith(Arith::Add(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xC6 => Ok((
                Instruction::Arith(Arith::Add(Operand::Immediate(bytes[1]))),
                2,
            )),

            0x88..=0x8F => Ok((
                Instruction::Arith(Arith::AddWithCarry(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xCE => Ok((
                Instruction::Arith(Arith::AddWithCarry(Operand::Immediate(bytes[1]))),
                2,
            )),

            0x90..=0x97 => Ok((
                Instruction::Arith(Arith::Subtract(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xD6 => Ok((
                Instruction::Arith(Arith::Subtract(Operand::Immediate(bytes[1]))),
                2,
            )),

            0x98..=0x9F => Ok((
                Instruction::Arith(Arith::SubtractWithCarry(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xDE => Ok((
                Instruction::Arith(Arith::SubtractWithCarry(Operand::Immediate(bytes[1]))),
                2,
            )),

            0xE6 => Ok((Instruction::Logic(Logic::AndImmediate(bytes[1])), 2)),
            0xA6 => Ok((Instruction::Logic(Logic::AndIndirect), 1)),

            0xA0 => Ok((Instruction::Logic(Logic::AndRegister(Register8::B)), 1)),
            0xA1 => Ok((Instruction::Logic(Logic::AndRegister(Register8::C)), 1)),
            0xA2 => Ok((Instruction::Logic(Logic::AndRegister(Register8::D)), 1)),
            0xA3 => Ok((Instruction::Logic(Logic::AndRegister(Register8::E)), 1)),
            0xA4 => Ok((Instruction::Logic(Logic::AndRegister(Register8::H)), 1)),
            0xA5 => Ok((Instruction::Logic(Logic::AndRegister(Register8::L)), 1)),
            0xA7 => Ok((Instruction::Logic(Logic::AndRegister(Register8::A)), 1)),

            0xF6 => Ok((Instruction::Logic(Logic::OrImmediate(bytes[1])), 2)),
            0xB6 => Ok((Instruction::Logic(Logic::OrIndirect), 1)),

            0xB0 => Ok((Instruction::Logic(Logic::OrRegister(Register8::B)), 1)),
            0xB1 => Ok((Instruction::Logic(Logic::OrRegister(Register8::C)), 1)),
            0xB2 => Ok((Instruction::Logic(Logic::OrRegister(Register8::D)), 1)),
            0xB3 => Ok((Instruction::Logic(Logic::OrRegister(Register8::E)), 1)),
            0xB4 => Ok((Instruction::Logic(Logic::OrRegister(Register8::H)), 1)),
            0xB5 => Ok((Instruction::Logic(Logic::OrRegister(Register8::L)), 1)),
            0xB7 => Ok((Instruction::Logic(Logic::OrRegister(Register8::A)), 1)),

            0xEE => Ok((Instruction::Logic(Logic::XorImmediate(bytes[1])), 2)),
            0xAE => Ok((Instruction::Logic(Logic::XorIndirect), 1)),

            0xA8 => Ok((Instruction::Logic(Logic::XorRegister(Register8::B)), 1)),
            0xA9 => Ok((Instruction::Logic(Logic::XorRegister(Register8::C)), 1)),
            0xAA => Ok((Instruction::Logic(Logic::XorRegister(Register8::D)), 1)),
            0xAB => Ok((Instruction::Logic(Logic::XorRegister(Register8::E)), 1)),
            0xAC => Ok((Instruction::Logic(Logic::XorRegister(Register8::H)), 1)),
            0xAD => Ok((Instruction::Logic(Logic::XorRegister(Register8::L)), 1)),
            0xAF => Ok((Instruction::Logic(Logic::XorRegister(Register8::A)), 1)),

            0xC9 => Ok((Instruction::Control(Control::Return), 1)),
            0xD9 => Ok((Instruction::Control(Control::InterruptReturn), 1)),

            0xC0 | 0xD0 | 0xC8 | 0xD8 => Ok((
                Instruction::Control(Control::ReturnConditional(ConditionCode::from_bits(
                    bytes[0],
                ))),
                1,
            )),

            0x17 => Ok((Instruction::Bits(Bits::RotateLeftAccumulator), 1)),
            0x1F => Ok((Instruction::Bits(Bits::RotateRightAccumulator), 1)),
            0x07 => Ok((Instruction::Bits(Bits::RotateLeftCarryAccumulator), 1)),
            0x0F => Ok((Instruction::Bits(Bits::RotateRightCarryAccumulator), 1)),

            0xCB => match bytes[1] {
                0x00..=0x07 => Ok((
                    Instruction::Bits(Bits::RotateLeftCarry(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x08..=0x0F => Ok((
                    Instruction::Bits(Bits::RotateRightCarry(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x10..=0x17 => Ok((
                    Instruction::Bits(Bits::RotateLeft(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x18..=0x1F => Ok((
                    Instruction::Bits(Bits::RotateRight(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x20..=0x27 => Ok((
                    Instruction::Bits(Bits::ShiftLeftArithmetic(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x28..=0x2F => Ok((
                    Instruction::Bits(Bits::ShiftRightArithmetic(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x30..=0x37 => Ok((
                    Instruction::Bits(Bits::Swap(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x38..=0x3F => Ok((
                    Instruction::Bits(Bits::ShiftRightLogical(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x40..=0x7F => Ok((
                    Instruction::Bits(Bits::GetBit(
                        get_bits_bit(bytes[1]),
                        Operand::from_bits(bytes[1], 0),
                    )),
                    2,
                )),
                0x80..=0xBF => Ok((
                    Instruction::Bits(Bits::ResetBit(
                        get_bits_bit(bytes[1]),
                        Operand::from_bits(bytes[1], 0),
                    )),
                    2,
                )),
                0xC0..=0xFF => Ok((
                    Instruction::Bits(Bits::SetBit(
                        get_bits_bit(bytes[1]),
                        Operand::from_bits(bytes[1], 0),
                    )),
                    2,
                )),
            },
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                error!(
                    "Unknown instruction {:#X} {:#X} {:#X}",
                    bytes[0], bytes[1], bytes[2]
                );
                Err(())
            }
        }
    }
}

fn get_bits_bit(i: u8) -> u8 {
    (i >> 3) & 0b111
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::EnableInterrupts => write!(f, "ei"),
            Instruction::DisableInterrupts => write!(f, "di"),
            Instruction::Stop => write!(f, "stop"),
            Instruction::Halt => write!(f, "halt"),
            Instruction::SetCarry => write!(f, "scf"),
            Instruction::ClearCarry => write!(f, "ccf"),
            Instruction::Compare(o) => write!(f, "cp {}", o),
            Instruction::Arith(a) => a.fmt(f),
            Instruction::Bits(b) => b.fmt(f),
            Instruction::Load(l) => l.fmt(f),
            Instruction::Control(c) => c.fmt(f),
            Instruction::Logic(l) => l.fmt(f),
        }
    }
}
