use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MilestoneApproval {
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub initial_payment: u64,
    pub threshold: u16,
    #[max_len(2)]
    pub approved_by: Vec<Pubkey>,
    pub approvals: u16,
    pub is_signed: bool,
    pub multisig_bump: u8,
}
