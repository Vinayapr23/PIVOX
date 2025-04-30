use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use crate::state::{VaultAccount, Contract};

#[derive(Accounts)]
pub struct VaultDeposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_ata: Account<'info, TokenAccount>,

    /// CHECK
    #[account(mut)]
    pub client: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub freelancer: AccountInfo<'info>,

    pub usdc_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault_account", client.key().as_ref(), freelancer.key().as_ref()],
        bump = vault_account.vault_bump,
    )]
    pub vault_account: Account<'info, VaultAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"contract", client.key().as_ref(), freelancer.key().as_ref()],
        bump = contract.contract_bump
    )]
    pub contract: Account<'info, Contract>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> VaultDeposit<'info> {
   /*  pub fn deposit_funds(&mut self, amount: u64) -> Result<()> {
        // Transfer funds from depositor to vault ATA
        let cpi_accounts = Transfer {
            from: self.depositor_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.depositor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, amount)?;

        // Calculate new balance
        let new_balance = self
            .vault_account
            .balance
            .checked_add(amount)
            .ok_or_else(|| error!(VaultDepositError::Overflow))?;

        // Sum total required milestone funding
        let total_required: u64 = self
            .contract
            .milestones
            .iter()
            .map(|m| m.amount)
            .sum();

        // Enforce sufficient deposit
        require!(
            new_balance >= total_required,
            VaultDepositError::InsufficientFunds
        );

        // Update vault balance
        self.vault_account.balance = new_balance;
        Ok(())
    }*/


    pub fn deposit_funds(&mut self, amount: u64) -> Result<()> {
        // Sum total required milestone funding
        let total_required: u64 = self
            .contract
            .milestones
            .iter()
            .map(|m| m.amount)
            .sum();
    
        // Check that current vault balance is less than required
        require!(
            self.vault_account.balance < total_required,
            VaultDepositError::InsufficientFunds
        );
    
        // Prevent over-depositing
        require!(
            self.vault_account
                .balance
                .checked_add(amount)
                .ok_or_else(|| error!(VaultDepositError::Overflow))?
                <= total_required,
            VaultDepositError::Overflow
        );
    
        // Transfer funds from depositor to vault ATA
        let cpi_accounts = Transfer {
            from: self.depositor_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.depositor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, amount)?;
    
        // Update vault balance after successful transfer
        self.vault_account.balance = self
            .vault_account
            .balance
            .checked_add(amount)
            .ok_or_else(|| error!(VaultDepositError::Overflow))?;
    
        Ok(())
    }
    






}

#[error_code]
pub enum VaultDepositError {
    #[msg("Balance overflow")]
    Overflow,
    #[msg("Insufficient funds for milestones")]
    InsufficientFunds,
}
