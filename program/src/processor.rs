use std::convert::TryInto;

use crate::instruction::AppInstruction;
use crate::schema::lottery::MAX_PARTICIPANT;
use crate::{error::AppError, schema::lottery::Lottery};
use solana_program::native_token::{sol_to_lamports, LAMPORTS_PER_SOL};
use solana_program::program::invoke;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_pack::Pack,
    pubkey::Pubkey,
};
use solana_program::{msg, system_instruction};

// hack, Solana doesn't have random num gen
// warn. if you call this func within a sec it will return same number
// in our case, we only call this func for one time so the number will be random between 0..upto
// or you can generate random number off-chain and send it to program in instruction data
fn rand(upto: u8) -> usize {
    let now = Clock::get().unwrap().unix_timestamp as u64;

    (now % upto as u64).try_into().unwrap()
}

pub struct Processor {}

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AppInstruction::unpack(instruction_data)?;
        match instruction {
            AppInstruction::Init {
                entry_fees,
                initializers_commission,
            } => {
                let accounts_iter = &mut accounts.iter();
                let initializer_account = next_account_info(accounts_iter)?;
                let lottery_account = next_account_info(accounts_iter)?;

                if lottery_account.owner != program_id {
                    return Err(AppError::IncorrectProgramId.into());
                }

                if !initializer_account.is_signer {
                    return Err(AppError::InvalidInstruction.into());
                }

                let mut data = Lottery::unpack(&lottery_account.data.borrow())?;
                data.entry_fees = entry_fees;
                data.initializers_commission = initializers_commission;
                data.initializer = initializer_account.key.to_bytes();
                data.participants = [[0; 32]; MAX_PARTICIPANT];

                Lottery::pack(data, &mut lottery_account.data.borrow_mut())?;
                Ok(())
            }
            AppInstruction::Participate => {
                let accounts_iter = &mut accounts.iter();
                let participant_account = next_account_info(accounts_iter)?;
                let lottery_account = next_account_info(accounts_iter)?;
                let sys_program = next_account_info(accounts_iter)?;

                if lottery_account.owner != program_id {
                    return Err(AppError::IncorrectProgramId.into());
                }

                if !participant_account.is_signer {
                    return Err(AppError::InvalidInstruction.into());
                }

                let mut data = Lottery::unpack(&lottery_account.data.borrow())?;
                let mut participants = [[0; 32]; MAX_PARTICIPANT];

                {
                    let mut index = 0;
                    let entry_amount = (data.entry_fees as u64) * LAMPORTS_PER_SOL;

                    for p in data.participants.iter() {
                        if p.ne(&[0; 32]) {
                            if p.eq(&participant_account.key.to_bytes()) {
                                // if participant already in Lottery
                                return Err(AppError::DuplicateEntry.into());
                            }

                            participants[index] = *p;
                        } else {
                            participants[index] = participant_account.key.to_bytes();
                            if participant_account.lamports() < entry_amount {
                                // low balance to enter in Lottery
                                return Err(AppError::LowBalance.into());
                            }

                            // transfer sol to program
                            let tr_ix = system_instruction::transfer(
                                participant_account.key,
                                lottery_account.key,
                                entry_amount,
                            );

                            invoke(
                                &tr_ix,
                                &[
                                    participant_account.clone(),
                                    lottery_account.clone(),
                                    sys_program.clone(),
                                ],
                            )?;

                            // cant we use try_borrow_mut_lamports() instead of cpi (invoking transfer instruction)?
                            // Nope, we cant reduce account balance that is not own by the program.

                            break;
                        }

                        index += 1;
                    }

                    if index == MAX_PARTICIPANT {
                        // Lottery already full
                        return Err(AppError::NoRoom.into());
                    }
                }

                data.participants = participants;
                Lottery::pack(data, &mut lottery_account.data.borrow_mut())?;

                Ok(())
            }
            AppInstruction::PickWinner => {
                let accounts_iter = &mut accounts.iter();
                let initializer_account = next_account_info(accounts_iter)?;
                let lottery_account = next_account_info(accounts_iter)?;

                if lottery_account.owner != program_id {
                    return Err(AppError::IncorrectProgramId.into());
                }

                if !initializer_account.is_signer {
                    return Err(AppError::MustSigner.into());
                }

                let data = Lottery::unpack(&lottery_account.data.borrow())?;

                if initializer_account.key.to_bytes() != data.initializer {
                    // unauthorized
                    return Err(AppError::Unauthorized.into());
                }

                if data.participants[data.participants.len() - 1].eq(&[0; 32]) {
                    // Lottery is not filled with all participants
                    return Err(AppError::EmptyRoom.into());
                }

                let winner_index: usize = rand(data.participants.len().try_into().unwrap());
                let winner = data.participants[winner_index];

                let mut winner_account: Option<&AccountInfo> = None;

                loop {
                    match next_account_info(accounts_iter) {
                        Ok(account) => {
                            if account.key.to_bytes().eq(&winner) {
                                winner_account = Some(account);
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }

                if winner_account.is_none() {
                    // invalid participants accounts
                    return Err(AppError::IncorrectProgramId.into());
                }

                msg!("Lottery winner is: {:?}", winner_account.unwrap().key);

                let account_lamports = lottery_account.lamports();
                let lottery_amount =
                    sol_to_lamports((data.participants.len() as f64) * (data.entry_fees as f64));

                // lottery initializer will get this initializers_commission entry_fees as commission for organizing the lottery from winner
                let fees_amount = lottery_amount * (data.initializers_commission as u64) / 100;
                let remaining_amount = account_lamports - lottery_amount;

                // transfer lottery entry_fees to winner
                **lottery_account.try_borrow_mut_lamports()? -= account_lamports;
                **winner_account.unwrap().try_borrow_mut_lamports()? +=
                    lottery_amount - fees_amount;

                // transfer remaining entry_fees to lottery initializer
                **initializer_account.try_borrow_mut_lamports()? += remaining_amount + fees_amount;

                // clear account data
                *lottery_account.data.borrow_mut() = &mut [];

                Ok(())
            }
        }
    }
}
