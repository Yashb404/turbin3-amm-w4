use {
    anchor_lang::{Id, InstructionData, ToAccountMetas},
    anchor_spl::{associated_token::AssociatedToken, token::Token},
    anchor_lang::prelude::System,
    solana_message::Instruction,
    solana_pubkey::Pubkey,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

pub fn create_initialize_ix(
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    mint_lp: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
) -> Instruction {
    let maker = payer.pubkey();

    Instruction::new_with_bytes(
        amm::id(),
        &amm::instruction::Initialize {
            seed: 0,
            fee: 30,
            authority: Some(maker),
        }
        .data(),
        amm::accounts::Initialize {
            initializer: maker,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            token_program: Token::id(),
            associated_token_program: AssociatedToken::id(),
            system_program: System::id(),
        }
        .to_account_metas(None),
    )
}
