export const BasePaintSubscriptionAbi = [
  { inputs: [{ internalType: "address", name: "target", type: "address" }], name: "AddressEmptyCode", type: "error" },
  { inputs: [], name: "DayCountMismatch", type: "error" },
  {
    inputs: [
      { internalType: "address", name: "sender", type: "address" },
      { internalType: "uint256", name: "balance", type: "uint256" },
      { internalType: "uint256", name: "needed", type: "uint256" },
      { internalType: "uint256", name: "tokenId", type: "uint256" },
    ],
    name: "ERC1155InsufficientBalance",
    type: "error",
  },
  {
    inputs: [{ internalType: "address", name: "approver", type: "address" }],
    name: "ERC1155InvalidApprover",
    type: "error",
  },
  {
    inputs: [
      { internalType: "uint256", name: "idsLength", type: "uint256" },
      { internalType: "uint256", name: "valuesLength", type: "uint256" },
    ],
    name: "ERC1155InvalidArrayLength",
    type: "error",
  },
  {
    inputs: [{ internalType: "address", name: "operator", type: "address" }],
    name: "ERC1155InvalidOperator",
    type: "error",
  },
  {
    inputs: [{ internalType: "address", name: "receiver", type: "address" }],
    name: "ERC1155InvalidReceiver",
    type: "error",
  },
  {
    inputs: [{ internalType: "address", name: "sender", type: "address" }],
    name: "ERC1155InvalidSender",
    type: "error",
  },
  {
    inputs: [
      { internalType: "address", name: "operator", type: "address" },
      { internalType: "address", name: "owner", type: "address" },
    ],
    name: "ERC1155MissingApprovalForAll",
    type: "error",
  },
  {
    inputs: [{ internalType: "address", name: "implementation", type: "address" }],
    name: "ERC1967InvalidImplementation",
    type: "error",
  },
  { inputs: [], name: "ERC1967NonPayable", type: "error" },
  { inputs: [], name: "FailedInnerCall", type: "error" },
  { inputs: [], name: "InvalidInitialization", type: "error" },
  { inputs: [], name: "InvalidSubscribedDay", type: "error" },
  { inputs: [], name: "NotInitializing", type: "error" },
  { inputs: [{ internalType: "address", name: "owner", type: "address" }], name: "OwnableInvalidOwner", type: "error" },
  {
    inputs: [{ internalType: "address", name: "account", type: "address" }],
    name: "OwnableUnauthorizedAccount",
    type: "error",
  },
  { inputs: [], name: "UUPSUnauthorizedCallContext", type: "error" },
  {
    inputs: [{ internalType: "bytes32", name: "slot", type: "bytes32" }],
    name: "UUPSUnsupportedProxiableUUID",
    type: "error",
  },
  { inputs: [], name: "WrongEthAmount", type: "error" },
  {
    anonymous: false,
    inputs: [
      { indexed: true, internalType: "address", name: "account", type: "address" },
      { indexed: true, internalType: "address", name: "operator", type: "address" },
      { indexed: false, internalType: "bool", name: "approved", type: "bool" },
    ],
    name: "ApprovalForAll",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [{ indexed: false, internalType: "uint64", name: "version", type: "uint64" }],
    name: "Initialized",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      { indexed: true, internalType: "address", name: "previousOwner", type: "address" },
      { indexed: true, internalType: "address", name: "newOwner", type: "address" },
    ],
    name: "OwnershipTransferred",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      { indexed: true, internalType: "address", name: "operator", type: "address" },
      { indexed: true, internalType: "address", name: "from", type: "address" },
      { indexed: true, internalType: "address", name: "to", type: "address" },
      { indexed: false, internalType: "uint256[]", name: "ids", type: "uint256[]" },
      { indexed: false, internalType: "uint256[]", name: "values", type: "uint256[]" },
    ],
    name: "TransferBatch",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      { indexed: true, internalType: "address", name: "operator", type: "address" },
      { indexed: true, internalType: "address", name: "from", type: "address" },
      { indexed: true, internalType: "address", name: "to", type: "address" },
      { indexed: false, internalType: "uint256", name: "id", type: "uint256" },
      { indexed: false, internalType: "uint256", name: "value", type: "uint256" },
    ],
    name: "TransferSingle",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      { indexed: false, internalType: "string", name: "value", type: "string" },
      { indexed: true, internalType: "uint256", name: "id", type: "uint256" },
    ],
    name: "URI",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [{ indexed: true, internalType: "address", name: "implementation", type: "address" }],
    name: "Upgraded",
    type: "event",
  },
  {
    inputs: [],
    name: "UPGRADE_INTERFACE_VERSION",
    outputs: [{ internalType: "string", name: "", type: "string" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "account", type: "address" },
      { internalType: "uint256", name: "id", type: "uint256" },
    ],
    name: "balanceOf",
    outputs: [{ internalType: "uint256", name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address[]", name: "accounts", type: "address[]" },
      { internalType: "uint256[]", name: "ids", type: "uint256[]" },
    ],
    name: "balanceOfBatch",
    outputs: [{ internalType: "uint256[]", name: "", type: "uint256[]" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "basepaint",
    outputs: [{ internalType: "contract IBasePaint", name: "", type: "address" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "_basepaint", type: "address" },
      { internalType: "address", name: "_owner", type: "address" },
    ],
    name: "initialize",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "account", type: "address" },
      { internalType: "address", name: "operator", type: "address" },
    ],
    name: "isApprovedForAll",
    outputs: [{ internalType: "bool", name: "", type: "bool" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ internalType: "address[]", name: "_addresses", type: "address[]" }],
    name: "mintBasePaints",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "", type: "address" },
      { internalType: "address", name: "", type: "address" },
      { internalType: "uint256[]", name: "", type: "uint256[]" },
      { internalType: "uint256[]", name: "", type: "uint256[]" },
      { internalType: "bytes", name: "", type: "bytes" },
    ],
    name: "onERC1155BatchReceived",
    outputs: [{ internalType: "bytes4", name: "", type: "bytes4" }],
    stateMutability: "pure",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "", type: "address" },
      { internalType: "address", name: "", type: "address" },
      { internalType: "uint256", name: "", type: "uint256" },
      { internalType: "uint256", name: "", type: "uint256" },
      { internalType: "bytes", name: "", type: "bytes" },
    ],
    name: "onERC1155Received",
    outputs: [{ internalType: "bytes4", name: "", type: "bytes4" }],
    stateMutability: "pure",
    type: "function",
  },
  {
    inputs: [],
    name: "owner",
    outputs: [{ internalType: "address", name: "", type: "address" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "proxiableUUID",
    outputs: [{ internalType: "bytes32", name: "", type: "bytes32" }],
    stateMutability: "view",
    type: "function",
  },
  { inputs: [], name: "renounceOwnership", outputs: [], stateMutability: "nonpayable", type: "function" },
  {
    inputs: [
      { internalType: "address", name: "from", type: "address" },
      { internalType: "address", name: "to", type: "address" },
      { internalType: "uint256[]", name: "ids", type: "uint256[]" },
      { internalType: "uint256[]", name: "values", type: "uint256[]" },
      { internalType: "bytes", name: "data", type: "bytes" },
    ],
    name: "safeBatchTransferFrom",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "from", type: "address" },
      { internalType: "address", name: "to", type: "address" },
      { internalType: "uint256", name: "id", type: "uint256" },
      { internalType: "uint256", name: "value", type: "uint256" },
      { internalType: "bytes", name: "data", type: "bytes" },
    ],
    name: "safeTransferFrom",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "operator", type: "address" },
      { internalType: "bool", name: "approved", type: "bool" },
    ],
    name: "setApprovalForAll",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "_mintToAddress", type: "address" },
      { internalType: "uint256[]", name: "_days", type: "uint256[]" },
      { internalType: "uint256[]", name: "_counts", type: "uint256[]" },
    ],
    name: "subscribe",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [{ internalType: "bytes4", name: "interfaceId", type: "bytes4" }],
    name: "supportsInterface",
    outputs: [{ internalType: "bool", name: "", type: "bool" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ internalType: "address", name: "newOwner", type: "address" }],
    name: "transferOwnership",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "newImplementation", type: "address" },
      { internalType: "bytes", name: "data", type: "bytes" },
    ],
    name: "upgradeToAndCall",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [{ internalType: "uint256", name: "", type: "uint256" }],
    name: "uri",
    outputs: [{ internalType: "string", name: "", type: "string" }],
    stateMutability: "view",
    type: "function",
  },
  { stateMutability: "payable", type: "receive" },
] as const;
