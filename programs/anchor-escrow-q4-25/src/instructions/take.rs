#![allow(unused_imports)]

use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    /// CHECK: The maker is verified indirectly via the escrow account's `has_one = maker` constraint (implicitly, since the escrow PDA is derived from the maker's key). We only need their account to receive the closed rent from the escrow.
    #[account(mut)]
    pub maker: AccountInfo<'info>,
    #[account( mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account( mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        close = taker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", escrow.maker.key().as_ref(), &escrow.seed.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    //  TODO: Implement Take Instruction
    //  Includes Deposit, Withdraw and Close Vault
    pub fn deposit(&mut self, deposit_amount: u64) -> Result<()> {
        let transfer_account = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_account);

        transfer_checked(cpi_ctx, deposit_amount, self.mint_b.decimals)
    }

    pub fn withdraw_and_close_vault(&mut self, withdraw_amount: u64) -> Result<()> {
        let transfer_account = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let binding = self.escrow.seed.to_le_bytes();
        let maker_binding = self.escrow.maker.key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            &maker_binding.as_ref(),
            &binding,
            &[self.escrow.bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_account,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, withdraw_amount, self.mint_a.decimals)?;

        let closeaccount = CloseAccount {
            account: self.vault.to_account_info(),
            authority: self.escrow.to_account_info(),
            destination: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            closeaccount,
            signer_seeds,
        );

        close_account(cpi_ctx)
    }
}
