use crate::{
    constants::{OFFER, REWARD_CENTER},
    errors::RewardCenterError,
    state::{Offer, RewardCenter},
};
use anchor_lang::prelude::{Result, *};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mtly_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::{AuctioneerDeposit, AuctioneerPublicBuy},
    program::AuctionHouse as AuctionHouseProgram,
    utils::assert_metadata_valid,
    AuctionHouse, Auctioneer,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateOfferParams {
    pub trade_state_bump: u8,
    pub escrow_payment_bump: u8,
    pub buyer_price: u64,
    pub token_size: u64,
}

#[derive(Accounts, Clone)]
#[instruction(create_offer_params: CreateOfferParams)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The Offer config account used for bids
    #[account(
        init,
        payer = wallet,
        space = Offer::size(),
        seeds = [
            OFFER.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref()
        ],
        bump
    )]
    pub offer: Box<Account<'info, Offer>>,

    /// CHECK: Validated in public_bid_logic.
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,

    /// CHECK: Validated in public_bid_logic.
    pub transfer_authority: UncheckedAccount<'info>,

    pub treasury_mint: Box<Account<'info, Mint>>,

    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: assertion with mtly_auction_house assert_metadata_valid
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = create_offer_params.escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified with has_one constraint on auction house account.
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

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

    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = authority,
        has_one = treasury_mint,
        has_one = auction_house_fee_account
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

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            treasury_mint.key().as_ref(),
            token_account.mint.as_ref(),
            create_offer_params.buyer_price.to_le_bytes().as_ref(),
            create_offer_params.token_size.to_le_bytes().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = create_offer_params.trade_state_bump
    )]
    buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateOffer>,
    CreateOfferParams {
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
        ..
    }: CreateOfferParams,
) -> Result<()> {
    let metadata = &ctx.accounts.metadata;
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let token_account = &ctx.accounts.token_account;
    let wallet = &ctx.accounts.wallet;
    let clock = Clock::get()?;
    let offer = &mut ctx.accounts.offer;

    let auction_house_key = auction_house.key();

    offer.reward_center = reward_center.key();
    offer.buyer = wallet.key();
    offer.metadata = metadata.key();
    offer.price = buyer_price;
    offer.token_size = token_size;
    offer.bump = *ctx
        .bumps
        .get(OFFER)
        .ok_or(RewardCenterError::BumpSeedNotInHashMap)?;
    offer.created_at = clock.unix_timestamp;

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    assert_metadata_valid(metadata, token_account)?;

    let deposit_accounts_ctx = CpiContext::new_with_signer(
        ctx.accounts.auction_house_program.to_account_info(),
        AuctioneerDeposit {
            wallet: ctx.accounts.wallet.to_account_info(),
            transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
            auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            payment_account: ctx.accounts.payment_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
        reward_center_signer_seeds,
    );

    mtly_auction_house::cpi::auctioneer_deposit(
        deposit_accounts_ctx,
        escrow_payment_bump,
        buyer_price,
    )?;

    let public_buy_accounts_ctx = CpiContext::new_with_signer(
        ctx.accounts.auction_house_program.to_account_info(),
        AuctioneerPublicBuy {
            wallet: ctx.accounts.wallet.to_account_info(),
            payment_account: ctx.accounts.payment_account.to_account_info(),
            transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            token_account: ctx.accounts.token_account.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
            ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
        reward_center_signer_seeds,
    );

    mtly_auction_house::cpi::auctioneer_public_buy(
        public_buy_accounts_ctx,
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
    )?;

    Ok(())
}
