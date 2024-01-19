import { createConfig } from "@ponder/core";
import { fallback, http } from "viem";

import { BasePaintBrushAbi } from "./abis/BasePaintBrushAbi";
import { BasePaintAbi } from "./abis/BasePaintAbi";

export default createConfig({
  networks: {
    base: {
      chainId: 8453,
      transport: fallback(
        process.env
          .PONDER_RPC_URLS_8453!.split(",")
          .map((url) => http(url.trim()))
      ),
    },
  },
  contracts: {
    BasePaintBrush: {
      abi: BasePaintBrushAbi,
      address: "0xD68fe5b53e7E1AbeB5A4d0A6660667791f39263a",
      network: "base",
      startBlock: 0x246523,
    },
    BasePaint: {
      abi: BasePaintAbi,
      address: "0xBa5e05cb26b78eDa3A2f8e3b3814726305dcAc83",
      network: "base",
      startBlock: 0x246523,
    },
  },
});
