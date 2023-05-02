#![cfg(feature = "test-bpf")]

pub mod reward_center_test;
use anchor_client::solana_sdk::{signature::Signer, transaction::Transaction};
use mtly_auction_house::pda::find_auction_house_address;
use mtly_reward_center::{pda::find_reward_center_address, reward_centers, state::*};

use mpl_testing_utils::solana::airdrop;
use solana_program_test::*;
use solana_sdk::{program_pack::Pack, signature::Keypair, system_instruction::create_account};

use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    instruction::{initialize_mint, mint_to_checked},
    native_mint,
    state::Mint,
};

#[tokio::test]
async fn edit_reward_center_success() {
    let program = reward_center_test::setup_program();
    let mut context = program.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();
    let wallet = context.payer.pubkey();
    let mint = native_mint::id();

    let (auction_house, _) = find_auction_house_address(&wallet, &mint);

    // Creating Rewards mint and token account
    let token_program = &spl_token::id();
    let reward_mint_authority_keypair = Keypair::new();
    let reward_mint_keypair = Keypair::new();

    let reward_mint_authority_pubkey = reward_mint_authority_keypair.pubkey();
    let reward_mint_pubkey = reward_mint_keypair.pubkey();
    let (reward_center, _) = find_reward_center_address(&auction_house);

    airdrop(
        &mut context,
        &reward_mint_authority_pubkey,
        reward_center_test::TEN_SOL,
    )
    .await
    .unwrap();

    // Assign account and rent
    let mint_account_rent = rent.minimum_balance(Mint::LEN);
    let allocate_reward_mint_space_ix = create_account(
        &reward_mint_authority_pubkey,
        &reward_mint_pubkey,
        mint_account_rent,
        Mint::LEN as u64,
        &token_program,
    );

    // Initialize rewards mint
    let init_rewards_reward_mint_ix = initialize_mint(
        &token_program,
        &reward_mint_pubkey,
        &reward_mint_authority_pubkey,
        Some(&reward_mint_authority_pubkey),
        9,
    )
    .unwrap();

    // Minting initial tokens to reward_center
    let reward_center_reward_token_account =
        get_associated_token_address(&reward_center, &reward_mint_pubkey);

    let mint_reward_tokens_ix = mint_to_checked(
        &token_program,
        &reward_mint_pubkey,
        &reward_center_reward_token_account,
        &reward_mint_authority_pubkey,
        &[],
        100_000_000_000,
        9,
    )
    .unwrap();

    let reward_center_params = reward_centers::create::CreateRewardCenterParams {
        reward_rules: RewardRules {
            mathematical_operand: PayoutOperation::Divide,
            seller_reward_payout_basis_points: 1000,
            payout_numeral: 5,
        },
    };

    let edit_reward_center_params = reward_centers::edit::EditRewardCenterParams {
        reward_rules: RewardRules {
            mathematical_operand: PayoutOperation::Multiple,
            seller_reward_payout_basis_points: 2000,
            payout_numeral: 10,
        },
    };

    let create_auction_house_accounts = mtly_auction_house_sdk::CreateAuctionHouseAccounts {
        treasury_mint: mint,
        payer: wallet,
        authority: wallet,
        fee_withdrawal_destination: wallet,
        treasury_withdrawal_destination: wallet,
        treasury_withdrawal_destination_owner: wallet,
    };
    let create_auction_house_data = mtly_auction_house_sdk::CreateAuctionHouseData {
        seller_fee_basis_points: 100,
        requires_sign_off: false,
        can_change_sale_price: false,
    };

    let create_auction_house_ix = mtly_auction_house_sdk::create_auction_house(
        create_auction_house_accounts,
        create_auction_house_data,
    );

    let create_reward_center_ix = mtly_reward_center_sdk::create_reward_center(
        mtly_reward_center_sdk::accounts::CreateRewardCenterAccounts {
            wallet,
            mint: reward_mint_keypair.pubkey(),
            auction_house_treasury_mint: mint,
            auction_house,
        },
        reward_center_params,
    );

    let edit_reward_center_ix = mtly_reward_center_sdk::edit_reward_center(
        wallet,
        auction_house,
        edit_reward_center_params,
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_auction_house_ix,
            allocate_reward_mint_space_ix,
            init_rewards_reward_mint_ix,
            create_reward_center_ix,
            mint_reward_tokens_ix,
            edit_reward_center_ix,
        ],
        Some(&wallet),
        &[
            &context.payer,
            &reward_mint_authority_keypair,
            &reward_mint_keypair,
        ],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    ()
}
