#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("C9C8FvNPrgWkkD5UUzjQFfj3k3Y5JRvMhsYpqUKfN1J5");

#[program]
pub mod crudapp {
    use super::*;


    pub fn create_journal_entry(ctx: Context<CreateEntry>, title: String, message: String)  -> Result<()>{
      let journal_entry = &mut ctx.accounts.journal_entry;
      journal_entry.owner = *ctx.accounts.owner.key;
      journal_entry.title = title;
      journal_entry.message = message;

      Ok(())
    }
    
    pub fn update_journal_entry(ctx: Context<UpdateEntry>, _title: String, message: String)  -> Result<()>{
      let journal_entry = &mut ctx.accounts.journal_entry;
      journal_entry.message = message;
      Ok(())
    }

    pub fn delete_journal_entry(_ctx: Context<DeleteEntry>, _title: String)  -> Result<()>{
      Ok(())
    }

}


#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreateEntry<'info>{
  #[account(
    init,
    seeds = [title.as_bytes(), owner.key().as_ref()],
    bump,
    payer = owner,
    space = 8 + JournalEntry::INIT_SPACE
  )]
  pub journal_entry: Account<'info, JournalEntry>,

  #[account(mut)]
  pub owner: Signer<'info>,

  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct UpdateEntry<'info>{
  #[account(
    mut,
    seeds = [title.as_bytes(), owner.key().as_ref()],
    bump,
    realloc = 8 + JournalEntry::INIT_SPACE,
    realloc::payer = owner,
    realloc::zero = false,
  )]
  pub journal_entry: Account<'info, JournalEntry>,
  #[account(mut)]
  pub owner: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteEntry<'info>{
  #[account(
    mut,
    seeds = [title.as_bytes(), owner.key().as_ref()],
    bump,
    close = owner,
  )]
  pub journal_entry: Account<'info, JournalEntry>,
  #[account(mut)]
  pub owner: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct JournalEntry {
  pub owner: Pubkey,
  #[max_len(32)]
  pub title: String,
  #[max_len(256)]
  pub message: String,
}
