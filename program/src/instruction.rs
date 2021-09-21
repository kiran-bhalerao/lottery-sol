use crate::error::AppError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
    Init {
        entry_fees: u32,
        initializers_commission: u8,
    },
    Participate,
    PickWinner,
}

impl AppInstruction {
    pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = instruction
            .split_first()
            .ok_or(AppError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let entry_fees = rest
                    .get(..4)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u32::from_le_bytes)
                    .ok_or(AppError::InvalidInstruction)?;

                let initializers_commission = rest
                    .get(4..5)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u8::from_le_bytes)
                    .ok_or(AppError::InvalidInstruction)?;

                Self::Init {
                    entry_fees,
                    initializers_commission,
                }
            }
            1 => Self::Participate,
            2 => Self::PickWinner,
            _ => return Err(AppError::InvalidInstruction.into()),
        })
    }
}
