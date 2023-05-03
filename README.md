## Motley Labs Reward Center (`rwdD3F6CgoCAoVaxcitXAeWRjQdiGc5AVABKCpQSMfd`)

A fork of Holaplex [Reward Center](https://github.com/holaplex/reward-center-program) to allow Motley Labs to iterate quickly on new functionality.

This fork is deployed at `rwdD3F6CgoCAoVaxcitXAeWRjQdiGc5AVABKCpQSMfd` and can be used by the public.

`mtly-reward-center` composes on top of [mtly-auction-house](https://github.com/motleylabs/metaplex-program-library/tree/motleylabs/auction-house).

A crate is available at crates.io: [![Crate][mtly-reward-center-img-long]][mtly-reward-center-crate]

A TypeScript API is available on NPM: [![NPM][mtly-reward-center-nimg-long]][mtly-reward-center-npm]

## Verification of on-chain binary

Steps to verify the on-chain binary:

```
cd program
anchor verify rwdD3F6CgoCAoVaxcitXAeWRjQdiGc5AVABKCpQSMfd
```

# Reward Center

A program that facilitates the payout of a spl token as rewards to buyer and sellers on the successful sell of an NFTs.

## Definitions

reward center - A decorator for a Metaplex Auction Houses that owns the treasury of tokens to payout and manage reward rules. It is also the auctioneer delegate registered with the auction house program.

reward - Is the payout of some amount of tokens from the reward center treasury to the buyer and seller for the sell of an NFT.

reward rules - There are currently 2 configuration options the authority of the reward center can adjust to reward payout for a sale. They are the seller reward payout basis points and payout divider.

payout divider - The amount to divide from the sale amount which will result in the number of tokens to payout to the buyer and the seller. For example, a divider of 2 will payout half the amount sale amount as tokens. Its important that the purchase and reward token use the same number of decimals to ensure the math aligns.

seller reward payout basis points - The ratio of rewards to be sent to the seller. The rest of the rewards are claimed by the buyer. For example, 5,000 basis points will result in a 50-50 split of rewards to the buyer and the seller.


## Approach

The authority of an auction house can create a reward center to accompany it so tokens can be distributed when sales are brokered by the auction house. 

The reward center is the auctioneer delegate for the auction house of the auction house program.

Listing, offer, and purchase instructions destined for auction house are proxied through the reward center program so they can be enriched with rewards. Accounts are initialized for the listing and offers to track information needed to execute sales with auction house.

The authority of the reward center can control which NFTs receive rewards based on their associations to a Metaplex Collection. Rewardable collections are the on-chain record governing eligibility of an NFT for rewards. 

Through the auctioneer delegate feature of Auction House the reward center PDA is given authority over listings and offers ensuring any cancel requests go through the reward center program for documenting state changes.

## Instructions

### Create Reward Center

The authority of an auction house creates a reward center and sets the reward rules.

### Update Reward Center

The authority of an auction house with a reward center adjusts its configuration (e.g. collection oracle, reward rules).

### Withdraw Reward Center Funds

The authority of a reward center can withdraw the tokens stored in reward center treasury.

### Create Listing

User puts an NFT up for sale through the reward center program. This results in a CPI call to the *sale* instruction of auction house. A listing record is generated to track sale order.

### Cancel Listing

User cancels their listing resulting in *cancel* CPI call to auction house and cancellation time saved on the listing.

### Update Listing

The owner of a listing adjusts the sale price of the NFT.

### Buy Listing

Facilitates the sale of an NFT without needing to create an offer account by CPI calls to auction house *deposit* *public_buy* and *execute_sale* respectively. It then distributes rewards to the buyer and seller based on the configure reward rules by the auction house authority.

### Create Offer

User places an offer on an NFT resulting in a *public_bid* CPI call to auction house and the creation of an offer account for the reward center. The amount of the offer is deducted from the user's wallet and placed in their escrow account.

### Cancel Offer

Users cancels their offer resulting in *cancel* CPI call to auction house and cancellation time saved on the offer. The amount of the offer is deducted from the user's escrow account and transferred back to the user's wallet.

### Accept Offer

Facilitates the sale of an NFT without requiring the seller to create a listing account and allowing to "accept" an outstanding offer, by CPI calls to auction house *sell* and *execute_sale* respectively. It then distributes rewards to the buyer and seller based on the configure reward rules by the auction house authority.

## Testing

In order to run program specs peform the following operations:

```shell
$ cd program
$ ./build.sh
$ ./test.sh
```

[mtly-reward-center-crate]:https://crates.io/crates/mtly-reward-center
[mtly-reward-center-img-long]:https://img.shields.io/crates/v/mtly-reward-center?label=crates.io%20%7C%20mtly-reward-center&logo=rust
[mtly-reward-center-img]:https://img.shields.io/crates/v/mtly-reward-center?logo=rust

[mtly-reward-center-npm]:https://www.npmjs.com/package/@motleylabs/mtly-reward-center
[mtly-reward-center-nimg-long]:https://img.shields.io/npm/v/@motleylabs/mtly-reward-center?label=npm%20%7C%20mtly-reward-center&logo=typescript
[mtly-reward-center-nimg]:https://img.shields.io/npm/v/@motleylabs/mtly-reward-center?logo=typescript
