use std::convert::TryInto;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};

// Define maximum number of participants, if u change this here also change on client side
pub const MAX_PARTICIPANT: usize = 2;

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lottery {
    pub entry_fees: u32, // Participant has to pay this much to enter in Lottery
    pub initializers_commission: u8, // Percentage Reward for Lottery Organizer from Winner
    pub initializer: [u8; 32], // Lottery Organizer, has to pay initial lamps for Lottery account creation
    pub participants: [[u8; 32]; MAX_PARTICIPANT], // Array of MAX_PARTICIPANT no of Participants pubkeys
}

//
// Implement Sealed trait
//
impl Sealed for Lottery {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Lottery {
    fn is_initialized(&self) -> bool {
        true
    }
}

//
// Implement Pack trait
//
impl Pack for Lottery {
    // Fixed length
    const LEN: usize = 4 + 1 + 32 + 32 * MAX_PARTICIPANT;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Lottery::LEN];
        let (entry_fees, initializers_commission, initializer, participants) =
            array_refs![src, 4, 1, 32, 32 * MAX_PARTICIPANT];

        let participants: [[u8; 32]; MAX_PARTICIPANT] = participants
            .chunks(32)
            .map(|slice| {
                let slice_array: [u8; 32] = (&slice[..32])
                    .try_into()
                    .expect("error:: slice with incorrect length");

                slice_array
            })
            .collect::<Vec<[u8; 32]>>()
            .try_into()
            .expect("error:: convert Vec to slice");

        Ok(Lottery {
            entry_fees: u32::from_le_bytes(*entry_fees),
            initializers_commission: u8::from_be_bytes(*initializers_commission),
            initializer: *initializer,
            participants,
        })
    }

    // Pack data from the data struct to [u8]
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Lottery::LEN];
        let (dst_amount, dst_fees, dst_initializer, dst_participants) =
            mut_array_refs![dst, 4, 1, 32, 32 * MAX_PARTICIPANT];

        let &Lottery {
            entry_fees,
            initializers_commission,
            initializer,
            participants,
        } = self;

        let participants: [u8; 32 * MAX_PARTICIPANT] = participants
            .iter()
            .flatten()
            .map(|s| *s)
            .collect::<Vec<u8>>()
            .try_into()
            .expect("error:: convert Vec to slice");

        *dst_amount = entry_fees.to_le_bytes();
        *dst_fees = initializers_commission.to_le_bytes();
        *dst_initializer = initializer;
        *dst_participants = participants;
    }
}
