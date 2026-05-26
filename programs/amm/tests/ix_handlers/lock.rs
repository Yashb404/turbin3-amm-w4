use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System}, anchor_spl::{associated_token::AssociatedToken, mint, token::Token}, solana_keypair::Keypair, solana_message::Instruction, solana_pubkey::Pubkey, solana_signer::Signer
};

pub fn lock(
    payer: &Keypair,
    config: Pubkey,
    lock: bool,
) -> Instruction {
    let signer = payer.pubkey();

    Instruction::new_with_bytes(
        amm::id(),
        &amm::instruction::Lock {
            lock,
        }
        .data(),
        amm::accounts::Lock {
            signer: signer,
            config,
        }
        .to_account_metas(None),
    )
}
