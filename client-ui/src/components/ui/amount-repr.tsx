import { ethers } from "ethers";
import logger from "../../lib/logger";
import BigNumber from "bignumber.js";

export type Props = {
  weiAmount: string;
  tokenSymbol?: string;
}

export default function AmountRepr({ weiAmount, tokenSymbol }: Props) {
  try {
    const bigNum = new BigNumber(weiAmount);
    const fullWeiString = bigNum.toFixed(0);
    const etherAmount = ethers.formatEther(fullWeiString);
    return <span>{etherAmount}{tokenSymbol}</span>;
  } catch (error) {
    logger.error("Invalid amount:", error);
    return <span>Invalid amount</span>;
  }
}
