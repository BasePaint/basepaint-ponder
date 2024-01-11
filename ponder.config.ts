import { createConfig } from "@ponder/core";
import { http } from "viem";

import { BasePaintBrushAbi } from "./abis/BasePaintBrushAbi";

export default createConfig({
  networks: {
    base: { chainId: 8453, transport: http(process.env.PONDER_RPC_URL_8453) },
  },
  contracts: {
    BasePaintBrush: {
      abi: BasePaintBrushAbi,
      address: "0xD68fe5b53e7E1AbeB5A4d0A6660667791f39263a",
      network: "base",
      startBlock: 2385188,
    },
  },
});
