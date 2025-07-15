import { createConfig } from "ponder";
import { fallback, http } from "viem";

import { BasePaintBrushAbi } from "./abis/BasePaintBrushAbi";
import { BasePaintAbi } from "./abis/BasePaintAbi";
import { BasePaintBrushEventsAbi } from "./abis/BasePaintBrushEventsAbi";
import { BasePaintWIPAbi } from "./abis/BasePaintWIPAbi";
import { BasePaintAnimationAbi } from "./abis/BasePaintAnimationAbi";
import { BasePaintMetadataRegistryAbi } from "./abis/BasePaintMetadataRegistry";
import { BasePaintSubscriptionAbi } from "./abis/BasePaintSubscriptionAbi";

const { PONDER_RPC_URLS_8453 } = process.env;

export default createConfig({
  chains: {
    base: {
      id: 8453,
      rpc: fallback(PONDER_RPC_URLS_8453!.split(",").map((url) => http(url.trim()))),
    },
  },
  database: {
    kind: "postgres",
  },
  contracts: {
    BasePaintBrush: {
      abi: BasePaintBrushAbi,
      address: "0xD68fe5b53e7E1AbeB5A4d0A6660667791f39263a",
      chain: "base",
      startBlock: 0x246523,
    },
    BasePaint: {
      abi: BasePaintAbi,
      address: "0xBa5e05cb26b78eDa3A2f8e3b3814726305dcAc83",
      chain: "base",
      startBlock: 0x246523,
    },
    BasePaintBrushEvents: {
      abi: BasePaintBrushEventsAbi,
      address: "0xb152f48F207d9D1C30Ff60d46E8cb8c1a5d00dEC",
      chain: "base",
      startBlock: 15849605,
    },
    BasePaintWIP: {
      abi: BasePaintWIPAbi,
      address: "0xE6249eAfdC9C8a809fE28a5213120B1860f9a75f",
      chain: "base",
      startBlock: 3457270,
    },
    BasePaintAnimation: {
      abi: BasePaintAnimationAbi,
      address: "0xC59F475122e914aFCf31C0a9E0A2274666135e4E",
      chain: "base",
      startBlock: 19092516,
    },
    BasePaintMetadataRegistry: {
      abi: BasePaintMetadataRegistryAbi,
      address: "0x5104482a2Ef3a03b6270D3e931eac890b86FaD01",
      chain: "base",
      startBlock: 21992569,
    },
    BasePaintSubscription: {
      abi: BasePaintSubscriptionAbi,
      address: "0x75CF063a65d361527180805b244bC51c1deAb075",
      chain: "base",
      startBlock: 30570812,
    },
  },
});
