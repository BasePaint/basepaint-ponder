export const BasePaintBrushEventsAbi = [
  {
    inputs: [
      {
        internalType: "contract IBasePaintBrush",
        name: "_basePaintBrush",
        type: "address",
      },
    ],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  { anonymous: false, inputs: [], name: "Deployed", type: "event" },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "uint256",
        name: "tokenId",
        type: "uint256",
      },
      {
        indexed: false,
        internalType: "uint256",
        name: "strength",
        type: "uint256",
      },
    ],
    name: "StrengthChanged",
    type: "event",
  },
  {
    inputs: [],
    name: "basePaintBrush",
    outputs: [
      { internalType: "contract IBasePaintBrush", name: "", type: "address" },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "uint256", name: "tokenId", type: "uint256" },
      { internalType: "uint256", name: "strength", type: "uint256" },
      { internalType: "uint256", name: "nonce", type: "uint256" },
      { internalType: "bytes", name: "signature", type: "bytes" },
    ],
    name: "upgrade",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "uint256[]", name: "tokenIds", type: "uint256[]" },
      { internalType: "uint256[]", name: "strengths", type: "uint256[]" },
      { internalType: "uint256[]", name: "nonces", type: "uint256[]" },
      { internalType: "bytes[]", name: "signatures", type: "bytes[]" },
    ],
    name: "upgradeMulti",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;
