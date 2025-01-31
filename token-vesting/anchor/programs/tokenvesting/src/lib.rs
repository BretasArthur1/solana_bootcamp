#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("coUnmi3oBUtwtd9fjeAvSsJssXh5A5xyPbhpewyzRVF");

#[program]
pub mod tokenvesting {
    use super::*;

    pub fn create_vesting_account(ctx: Context<CreateVestingAccount>, company_name: String) -> Result<()> {
        *ctx.accounts.vesting_account = VestingAccount {
            owner: ctx.accounts.payer.key(),
            token_mint: ctx.accounts.mint.key(),
            treasury_token_account: ctx.accounts.treasury_token_account.key(),
            company_name,
            treasury_bump: ctx.bumps.treasury_token_account,
            bump: ctx.bumps.vesting_account,
        };

        Ok(())
    }

    pub fn create_employee_account(ctx: Context<CreateEmployeeAccount>, start_time: i64, end_time: i64, cliff_time: i64, total_amount: u64) -> Result<()> {
      *ctx.accounts.employee_account = EmployeeAccount {
        beneficiary: ctx.accounts.beneficiary.key(),
        start_time,
        end_time,
        cliff_time,
        vesting_account: ctx.accounts.vesting_account.key(),
        total_amount,
        total_withdrawn: 0,
        bump: ctx.bumps.employee_account,
      };

      Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, company_name: String) -> Result<()> {
        let employee_account = &mut ctx.accounts.employee_account;
        let now = Clock::get()?.unix_timestamp;

        require!(now >= employee_account.cliff_time, ErrorCode::ClaimNotAvailable);

        let time_since_start = now.saturating_sub(employee_account.start_time);
        let total_vesting_time = employee_account.end_time.saturating_sub(employee_account.start_time);
        require!(total_vesting_time > 0, ErrorCode::InvalidVestingPeriod);

        let vested_amount = if now >= employee_account.end_time {
            employee_account.total_amount
        } else {
            employee_account.total_amount
                .checked_mul(time_since_start as u64)
                .and_then(|amount| amount.checked_div(total_vesting_time as u64))
                .ok_or(ErrorCode::CalculationOverflow)?
        };

        let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
        require!(claimable_amount > 0, ErrorCode::ClaimNotAvailable);

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.treasury_token_account.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.employee_token_account.to_account_info(),
                    authority: ctx.accounts.treasury_token_account.to_account_info(),
                },
                &[&[
                    b"vesting_treasury".as_ref(),
                    company_name.as_bytes(),
                    &[ctx.accounts.vesting_account.treasury_bump],
                ]],
            ),
            claimable_amount,
            ctx.accounts.token_mint.decimals,
        )?;

        employee_account.total_withdrawn = employee_account.total_withdrawn
            .checked_add(claimable_amount)
            .ok_or(ErrorCode::CalculationOverflow)?;

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
      init,
      payer = payer,
      space = 8 + VestingAccount::INIT_SPACE,
      seeds = [b"vesting".as_ref(), company_name.as_ref()],
      bump,
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
      init,
      token::mint = mint,
      token::authority = treasury_token_account,
      payer = payer,
      seeds = [b"vesting_treasury".as_ref(), company_name.as_bytes()],
      bump,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateEmployeeAccount<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  pub beneficiary: SystemAccount<'info>,

  #[account(
    has_one = owner,
  )]
  pub vesting_account: Account<'info, VestingAccount>, 

  #[account(
    init,
    payer = owner,
    space = 8 + EmployeeAccount::INIT_SPACE,
    seeds = [b"employee_vesting".as_ref(), vesting_account.key().as_ref(), beneficiary.key().as_ref()],
    bump,
  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info> {
  #[account(mut)]
  pub beneficiary: Signer<'info>,

  #[account(
    mut,
    seeds = [b"employee_vesting".as_ref(), vesting_account.key().as_ref(), beneficiary.key().as_ref()],
    bump = employee_account.bump,
    has_one = vesting_account,
    has_one = beneficiary,
  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  #[account(
    mut,
    seeds = [company_name.as_ref()],
    bump = vesting_account.bump,
    has_one = treasury_token_account,
    has_one = token_mint,
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  pub token_mint: InterfaceAccount<'info, Mint>,

  #[account(mut)]
  pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = beneficiary,
    associated_token::mint = token_mint,
    associated_token::authority = beneficiary,
    associated_token::token_program = token_program,
  )]
  pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

  pub token_program: Interface<'info, TokenInterface>,

  pub associated_token_program: Program<'info, AssociatedToken>,

  pub system_program: Program<'info, System>,
}


#[account]
#[derive(InitSpace)]
pub struct VestingAccount {
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub treasury_token_account: Pubkey,
    #[max_len(32)]
    pub company_name: String,
    pub treasury_bump: u8,
    pub bump: u8,
}


#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount{
  pub beneficiary: Pubkey,
  pub start_time: i64,
  pub end_time: i64,
  pub cliff_time: i64,
  pub vesting_account: Pubkey,
  pub total_amount: u64,
  pub total_withdrawn: u64,
  pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
  #[msg("Claim not available")]
  ClaimNotAvailable,
  #[msg("Invalid vesting period")]
  InvalidVestingPeriod,
  #[msg("Calculation overflow")]
  CalculationOverflow,
}
