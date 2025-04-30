use anchor_lang::prelude::*;
use crate::state::milestone_approval::MilestoneApproval;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    /// CHECK: freelancer passed from frontend
    pub freelancer: AccountInfo<'info>,

    #[account(
        init,
        payer = client,
        space = 8 + MilestoneApproval::INIT_SPACE,
        seeds = [b"milestone_approval", client.key().as_ref(), freelancer.key().as_ref()],
        bump,
    )]
    pub milestone_approval: Account<'info, MilestoneApproval>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(
        &mut self,
        client: Pubkey,
        freelancer: Pubkey,
        initial_payment: u64,
        mut approved_by: Vec<Pubkey>,
        approvals: u16,
        is_signed: bool,
        threshold: u16,
        bump: u8,
    ) -> Result<()> {
        if !approved_by.contains(&client) {
            approved_by.push(client);
        }

        self.milestone_approval.set_inner(MilestoneApproval {
            client,
            freelancer,
            initial_payment,
            threshold,
            approved_by,
            approvals: approvals + 1,
            is_signed,
            multisig_bump: bump,
        });

        Ok(())
    }
}
