use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer}};
use crate::state::{Contract, VaultAccount, MilestoneApproval, Milestone};

#[derive(Accounts)]
pub struct MilestoneSignature<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,

    /// CHECK
    pub client: AccountInfo<'info>,

    pub usdc_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"milestone_approval", client.key().as_ref(), freelancer.key().as_ref()],
        bump
       
    )]
    pub milestone_approval: Account<'info, MilestoneApproval>,

    #[account(
        init,
        payer = freelancer,
        space = 8 + VaultAccount::INIT_SPACE,
        seeds = [b"vault_account", client.key().as_ref(), freelancer.key().as_ref()],
        bump
    )]
    pub vault_account: Account<'info, VaultAccount>,

    #[account(
        init,
        payer = freelancer,
        space = 8 + Contract::INIT_SPACE,
        seeds = [b"contract", client.key().as_ref(), freelancer.key().as_ref()],
        bump
    )]
    pub contract: Account<'info, Contract>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = freelancer,
    )]
    pub freelancer_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = client,
    )]
    pub client_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> MilestoneSignature<'info> {
    pub fn approve(
        &mut self,
        client_share: u8,
        freelancer_share: u8,
        initial_payment: u64,
        project_start: i128,
        project_duration: u64,
        dispute_resolution: String,
        vault_bump: u8,
        contract_bump: u8,
        vault_status: String,
        milestones: Vec<Milestone>,
    ) -> Result<()> {
        let milestone_approval = &mut self.milestone_approval;
        let freelancer_key = self.freelancer.key();

        require!(
            !milestone_approval.approved_by.contains(&freelancer_key),
            ErrorCode::AlreadyApproved
        );
        require!(
            !milestone_approval.is_signed,
            ErrorCode::AlreadySigned
        );

        milestone_approval.approved_by.push(freelancer_key);
        milestone_approval.approvals += 1;

        if milestone_approval.approvals == milestone_approval.threshold {
            self.finalize(
                client_share,
                freelancer_share,
                initial_payment,
                project_start,
                project_duration,
                dispute_resolution,
                vault_bump,
                contract_bump,
                vault_status,
                milestones,
            )?;
        }

        Ok(())
    }

    pub fn finalize(
        &mut self,
        client_share: u8,
        freelancer_share: u8,
        initial_payment: u64,
        project_start: i128,
        project_duration: u64,
        dispute_resolution: String,
        vault_bump: u8,
        contract_bump: u8,
        vault_status: String,
        milestones: Vec<Milestone>,
    ) -> Result<()> {
        self.milestone_approval.is_signed = true;

        self.vault_account.set_inner(VaultAccount {
            client: self.client.key(),
            freelancer: self.freelancer.key(),
            balance: 0,
            client_share,
            freelancer_share,
            multisig_account: self.milestone_approval.key(),
            vault_bump,
            vault_status,
        });

        self.contract.set_inner(Contract {
            client: self.client.key(),
            freelancer: self.freelancer.key(),
            vault_account: self.vault_account.key(),
            multisig_account: self.milestone_approval.key(),
            initial_payment,
            project_start,
            project_duration,
            client_share,
            freelancer_share,
            dispute_resolution,
            status: "active".to_string(),
            contract_bump,
            client_approved: false,
            freelancer_approved: false,
            milestones,
        });


        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Already Approved")]
    AlreadyApproved,
    #[msg("Already Signed")]
    AlreadySigned,
}
