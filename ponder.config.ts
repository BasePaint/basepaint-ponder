import { createConfig } from "@ponder/core";
import { fallback, http } from "viem";

import { BasePaintBrushAbi } from "./abis/BasePaintBrushAbi";
import { BasePaintAbi } from "./abis/BasePaintAbi";
import { BasePaintBrushEventsAbi } from "./abis/BasePaintBrushEventsAbi";
import { BasePaintWIPAbi } from "./abis/BasePaintWIPAbi";
import { BasePaintAnimationAbi } from "./abis/BasePaintAnimationAbi";

const { RAILWAY_HEALTHCHECK_TIMEOUT_SEC, PONDER_RPC_URLS_8453 } = process.env;

const maxHealthcheckDuration = (Number(RAILWAY_HEALTHCHECK_TIMEOUT_SEC ?? 55) - 1) * 60;

console.log({ RAILWAY_HEALTHCHECK_TIMEOUT_SEC, maxHealthcheckDuration });

export default createConfig({
  networks: {
    base: {
      chainId: 8453,
      transport: fallback(PONDER_RPC_URLS_8453!.split(",").map((url) => http(url.trim()))),
    },
  },
  options: {
    maxHealthcheckDuration,
  },
  database: {
    kind: "postgres",
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
    BasePaintBrushEvents: {
      abi: BasePaintBrushEventsAbi,
      address: "0xb152f48F207d9D1C30Ff60d46E8cb8c1a5d00dEC",
      network: "base",
      startBlock: 15849605,
    },
    BasePaintWIP: {
      abi: BasePaintWIPAbi,
      address: "0xE6249eAfdC9C8a809fE28a5213120B1860f9a75f",
      network: "base",
      startBlock: 3457270,
    },
    BasePaintAnimation: {
      abi: BasePaintAnimationAbi,
      address: "0xC59F475122e914aFCf31C0a9E0A2274666135e4E",
      network: "base",
      startBlock: 19092516,
    },
  },
});
