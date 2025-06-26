use std::str::FromStr;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use raydium_cp_swap::{
    cpi,
    program::RaydiumCpSwap,
    states::{AmmConfig},
};

use crate::{FEE_WALLET1};

#[derive(Accounts)]
pub struct MultiBuy<'info> {
    pub cp_swap_program: Program<'info, RaydiumCpSwap>,
    /// The user performing the swap
    pub payer: Signer<'info>,

    /// CHECK: the fee wallet 1 hardcoded
    #[account(
        mut,
        constraint = fee_wallet1.key() == Pubkey::from_str(FEE_WALLET1).unwrap()
    )]
    pub fee_wallet1: UncheckedAccount<'info>,

    /// CHECK: the fee wallet 2 hardcoded
    // #[account(
    //     mut,
    //     constraint = fee_wallet2.key() == Pubkey::from_str(FEE_WALLET2).unwrap()
    // )]
    // pub fee_wallet2: UncheckedAccount<'info>,

    /// CHECK: the fee wallet 3 hardcoded
    // #[account(
    //     mut,
    //     constraint = fee_wallet3.key() == Pubkey::from_str(FEE_WALLET3).unwrap()
    // )]
    // pub fee_wallet3: UncheckedAccount<'info>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            raydium_cp_swap::AUTH_SEED.as_bytes(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    // #[account(address = pool_state_1.load()?.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,

    // Per-token accounts (repeat for 12)
    #[account(mut)]
    pub wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// SPL program for input/output token transfers
    pub token_program: Interface<'info, TokenInterface>,

    /// The mint of input token WSOL
    // #[account(
    //     address = input_vault_1.mint
    // )]
    pub wsol_mint: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum CustomError {
    #[msg("token_length must be between 1 and 12")]
    InvalidTokenLength,
}

pub fn multi_buy<'info>(
    ctx: Context<'_, '_, '_, 'info, MultiBuy<'info>>,
    amount_in: u64,
    minimum_amount_out: u64,
    token_length: u8,
) -> Result<()> {
    require!(
        token_length >= 1 && token_length <= 12,
        CustomError::InvalidTokenLength
    );

    // Transfer SOL from payer to wsol_account
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.wsol_account.to_account_info(),
            },
        ),
        amount_in * token_length as u64,
    )?;

    // Sync native token
    token::sync_native(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::SyncNative {
            account: ctx.accounts.wsol_account.to_account_info(),
        },
    ))?;

    // pool_states
    // output_token_accounts
    // input_vaults
    // output_vaults
    // output_token_mints
    // observation_states

    for _i in 0..(token_length as usize) {
        let base = _i * 6;
        let cpi_accounts = cpi::accounts::Swap {
            payer: ctx.accounts.payer.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            amm_config: ctx.accounts.amm_config.to_account_info(),
            pool_state: ctx.remaining_accounts.get(base).unwrap().to_account_info(),
            input_token_account: ctx.accounts.wsol_account.to_account_info(),
            output_token_account: ctx
                .remaining_accounts
                .get(base + 1)
                .unwrap()
                .to_account_info(),
            input_vault: ctx
                .remaining_accounts
                .get(base + 2)
                .unwrap()
                .to_account_info(),
            output_vault: ctx
                .remaining_accounts
                .get(base + 3)
                .unwrap()
                .to_account_info(),
            input_token_program: ctx.accounts.token_program.to_account_info(),
            output_token_program: ctx.accounts.token_program.to_account_info(),
            input_token_mint: ctx.accounts.wsol_mint.to_account_info(),
            output_token_mint: ctx
                .remaining_accounts
                .get(base + 4)
                .unwrap()
                .to_account_info(),
            observation_state: ctx
                .remaining_accounts
                .get(base + 5)
                .unwrap()
                .to_account_info(),
        };
        let cpi_context =
            CpiContext::new(ctx.accounts.cp_swap_program.to_account_info(), cpi_accounts);
        cpi::swap_base_input(cpi_context, amount_in, minimum_amount_out)?;
    }

    // Transfer fees as before, using total_amount = amount_in * token_length as u64
    let total_amount = amount_in * token_length as u64;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.fee_wallet1.to_account_info(),
            },
        ),
        total_amount * 10 / 1000,
    )?;
    // system_program::transfer(
    //     CpiContext::new(
    //         ctx.accounts.system_program.to_account_info(),
    //         system_program::Transfer {
    //             from: ctx.accounts.payer.to_account_info(),
    //             to: ctx.accounts.fee_wallet2.to_account_info(),
    //         },
    //     ),
    //     total_amount * 4 / 1000,
    // )?;
    // system_program::transfer(
    //     CpiContext::new(
    //         ctx.accounts.system_program.to_account_info(),
    //         system_program::Transfer {
    //             from: ctx.accounts.payer.to_account_info(),
    //             to: ctx.accounts.fee_wallet3.to_account_info(),
    //         },
    //     ),
    //     total_amount * 4 / 1000,
    // )?;

    Ok(())
}
