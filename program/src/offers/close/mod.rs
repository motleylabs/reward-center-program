use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mtly_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::{AuctioneerCancel, AuctioneerWithdraw},
    instruction::{
        AuctioneerCancel as AuctioneerCancelParams, AuctioneerWithdraw as AuctioneerWithdrawParams,
    },
    program::AuctionHouse as AuctionHouseProgram,
    utils::{assert_derivation, assert_metadata_valid},
    AuctionHouse, Auctioneer,
};
use solana_program::system_program;

use crate::{
    constants::{OFFER, REWARD_CENTER},
    errors::RewardCenterError,
    id,
    metaplex_cpi::auction_house::{make_auctioneer_instruction, AuctioneerInstructionArgs},
    state::{Offer, RewardCenter},
};
use solana_program::program::invoke_signed;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CloseOfferParams {
    pub escrow_payment_bump: u8,
}

#[derive(Accounts, Clone)]
#[instruction(close_offer_params: CloseOfferParams)]
pub struct CloseOffer<'info> {
    /// User wallet account.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The Offer config account used for bids
    /// CHECK: Seed check in close offer logic
    #[account(mut)]
    pub offer: UncheckedAccount<'info>,

    pub treasury_mint: Box<Account<'info, Mint>>,

    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Validated in auction house program withdraw_logic.
    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = close_offer_params.escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: assertion with mtly_auction_house assert_metadata_valid
    /// Metaplex metadata account decorating SPL mint account.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Token mint account of SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Verified with has_one constraint on auction house account.
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Verified in ah_auctioneer_pda seeds and in bid logic.
    /// The auctioneer authority - typically a PDA of the Auctioneer program running this action.
    #[account(
        has_one = auction_house,
        seeds = [
            REWARD_CENTER.as_bytes(),
            auction_house.key().as_ref()
        ],
        bump = reward_center.bump
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump =auction_house.bump,
        has_one=authority,
        has_one=auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Validated in auction house program cancel_logic.
    /// Trade state PDA account representing the bid or ask to be canceled.
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,

    /// CHECK: Validated in auction house program cancel_logic.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            reward_center.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = ah_auctioneer_pda.bump
    )]
    pub ah_auctioneer_pda: Box<Account<'info, Auctioneer>>,

    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CloseOffer>,
    CloseOfferParams {
        escrow_payment_bump,
    }: CloseOfferParams,
) -> Result<()> {
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let metadata = &ctx.accounts.metadata;
    let token_account = &ctx.accounts.token_account;
    let wallet = &ctx.accounts.wallet;
    let offer_unchecked = &ctx.accounts.offer;
    let offer_account_info = offer_unchecked.to_account_info();
    let offer = try_deserialize_offer(&offer_account_info)?;
    let token_size = offer.token_size;
    let buyer_price = offer.price;
    let price = offer.price_with_fees;
    let auction_house_key = auction_house.key();

    assert_metadata_valid(metadata, token_account)?;
    let offer_bump = assert_derivation(
        &id(),
        &offer_account_info,
        &[
            OFFER.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref(),
        ],
    )?;
    require_eq!(offer_bump, offer.bump, RewardCenterError::BumpMismatch);

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    // Withdraw offer transaction via invoke_signed
    let withdraw_offer_ctx_accounts = AuctioneerWithdraw {
        wallet: wallet.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        ata_program: ctx.accounts.ata_program.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        receipt_account: ctx.accounts.receipt_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
    };

    let withdraw_offer_params = AuctioneerWithdrawParams {
        escrow_payment_bump,
        amount: price,
    };

    let (withdraw_offer_ix, withdraw_offer_account_infos) =
        make_auctioneer_instruction(AuctioneerInstructionArgs {
            accounts: withdraw_offer_ctx_accounts,
            instruction_data: withdraw_offer_params.data(),
            auctioneer_authority: ctx.accounts.reward_center.key(),
            remaining_accounts: None,
        });

    invoke_signed(
        &withdraw_offer_ix,
        &withdraw_offer_account_infos,
        reward_center_signer_seeds,
    )?;

    // Cancel (Close Offer) instruction via invoke_signed
    let cancel_offer_ctx_accounts = AuctioneerCancel {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        trade_state: ctx.accounts.trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let close_offer_params = AuctioneerCancelParams {
        buyer_price,
        token_size,
    };

    let (cancel_offer_ix, cancel_offer_account_infos) =
        make_auctioneer_instruction(AuctioneerInstructionArgs {
            accounts: cancel_offer_ctx_accounts,
            instruction_data: close_offer_params.data(),
            auctioneer_authority: ctx.accounts.reward_center.key(),
            remaining_accounts: None,
        });

    invoke_signed(
        &cancel_offer_ix,
        &cancel_offer_account_infos,
        reward_center_signer_seeds,
    )?;

    let dest_starting_lamports = wallet.lamports();
    **wallet.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(offer_account_info.lamports())
        .unwrap();
    **offer_account_info.lamports.borrow_mut() = 0;

    offer_account_info.assign(&system_program::id());
    offer_account_info.realloc(0, false)?;

    Ok(())
}

fn try_deserialize_offer(offer: &AccountInfo) -> Result<Offer> {
    let offer_ref_data = offer.try_borrow_mut_data()?;
    let mut offer_data: &[u8] = &offer_ref_data;

    Offer::try_deserialize(&mut offer_data)
}
