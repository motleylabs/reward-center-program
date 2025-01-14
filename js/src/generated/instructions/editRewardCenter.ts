/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import {
  EditRewardCenterParams,
  editRewardCenterParamsBeet,
} from '../types/EditRewardCenterParams';

/**
 * @category Instructions
 * @category EditRewardCenter
 * @category generated
 */
export type EditRewardCenterInstructionArgs = {
  editRewardCenterParams: EditRewardCenterParams;
};
/**
 * @category Instructions
 * @category EditRewardCenter
 * @category generated
 */
export const editRewardCenterStruct = new beet.BeetArgsStruct<
  EditRewardCenterInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['editRewardCenterParams', editRewardCenterParamsBeet],
  ],
  'EditRewardCenterInstructionArgs',
);
/**
 * Accounts required by the _editRewardCenter_ instruction
 *
 * @property [_writable_, **signer**] wallet
 * @property [] auctionHouse
 * @property [_writable_] rewardCenter
 * @category Instructions
 * @category EditRewardCenter
 * @category generated
 */
export type EditRewardCenterInstructionAccounts = {
  wallet: web3.PublicKey;
  auctionHouse: web3.PublicKey;
  rewardCenter: web3.PublicKey;
  anchorRemainingAccounts?: web3.AccountMeta[];
};

export const editRewardCenterInstructionDiscriminator = [185, 238, 29, 159, 195, 37, 196, 120];

/**
 * Creates a _EditRewardCenter_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category EditRewardCenter
 * @category generated
 */
export function createEditRewardCenterInstruction(
  accounts: EditRewardCenterInstructionAccounts,
  args: EditRewardCenterInstructionArgs,
  programId = new web3.PublicKey('rwdD3F6CgoCAoVaxcitXAeWRjQdiGc5AVABKCpQSMfd'),
) {
  const [data] = editRewardCenterStruct.serialize({
    instructionDiscriminator: editRewardCenterInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.wallet,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.auctionHouse,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rewardCenter,
      isWritable: true,
      isSigner: false,
    },
  ];

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc);
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
