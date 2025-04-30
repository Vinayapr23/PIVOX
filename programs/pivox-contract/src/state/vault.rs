use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultAccount {
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub balance: u64,
    pub client_share: u8,
    pub freelancer_share: u8,
    pub multisig_account: Pubkey,
    pub vault_bump: u8,
    #[max_len(10)]
    pub vault_status: String,
}
