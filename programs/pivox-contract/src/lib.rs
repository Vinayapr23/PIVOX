pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("ENhCXGcdC1s1kYpGVWSiMkR8snaxrCzSSTjbW7ViDioG");

#[program]
pub mod pivox_contract {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, client: Pubkey, freelancer: Pubkey, initial_payment: u64, approved_by: Vec<Pubkey>, approvals: u16, is_signed: bool, threshold: u16, bump: u8) -> Result<()> {
        ctx.accounts.initialize(client, freelancer, initial_payment, approved_by, approvals, is_signed, threshold, bump)
    }

    
    pub fn initialize_milestone_approval(
        ctx: Context<InitializeMilestoneApproval>,
        threshold: u16,
    ) -> Result<()> {
        ctx.accounts.initialize(threshold)
    }

    pub fn approve(
        ctx: Context<MilestoneSignature>,
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
        ctx.accounts.approve(client_share, freelancer_share, initial_payment, project_start, project_duration, dispute_resolution, vault_bump, contract_bump, vault_status, milestones)
    }


    pub fn complete_or_cancel_contract(ctx: Context<ContractFunc>) -> Result<()> {
        ctx.accounts.complete_or_cancel_contract()
    }

    pub fn release_milestone_payment(ctx: Context<ContractFunc>, milestone_index: u64) -> Result<()> {
        ctx.accounts.release_milestone_payment(milestone_index)
    }

    

   // pub fn pause_vault(ctx: Context<VaultConfig>) -> Result<()> {
     // ctx.accounts.pause_vault()
     //}

    // pub fn restart_vault(ctx: Context<VaultConfig>) -> Result<()> {
    //     ctx.accounts.restart_vault()
    // }

   

    pub fn deposit_funds(ctx: Context<VaultDeposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit_funds(amount)
    }

    pub fn withdraw_funds(ctx: Context<VaultWithdraw>) -> Result<()> {
        ctx.accounts.withdraw_funds()
    }


    pub fn freelancer_submit_milestone(ctx: Context<MilestoneLifecycle>, milestone_index: u64) -> Result<()> {
        ctx.accounts.freelancer_submit(milestone_index)
    }

    pub fn client_approve_milestone(ctx: Context<MilestoneLifecycle>, milestone_index: u64) -> Result<()> {
        ctx.accounts.client_approve(milestone_index)
    }

    pub fn freelancer_confirm_milestone(ctx: Context<MilestoneLifecycle>, milestone_index: u64) -> Result<()> {
        ctx.accounts.freelancer_confirm(milestone_index)
    }






    
}
