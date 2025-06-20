import { onchainTable, index } from "ponder";

export const Global = onchainTable("global", (t) => ({
  id: t.integer().primaryKey(),
  startedAt: t.integer(),
  epochDuration: t.integer(),
  totalArtists: t.integer(),
  totalPixels: t.integer(),
  totalEarnings: t.bigint(),
  totalWithdrawals: t.bigint(),
  totalMints: t.integer(),
  totalBurns: t.integer(),
  totalSubscriptions: t.integer(),
}));

export const Account = onchainTable("account", (t) => ({
  id: t.text().primaryKey(),
  totalPixels: t.integer(),
  totalWithdrawn: t.bigint(),
  totalEarned: t.bigint(),
  streak: t.integer(),
  longestStreak: t.integer(),
  lastPaintedDay: t.integer(),
}));

export const Brush = onchainTable(
  "brush",
  (t) => ({
    id: t.integer().primaryKey(),
    ownerId: t.text().notNull(),
    strength: t.integer(),
    lastUsedTimestamp: t.integer(),
    lastUsedDay: t.integer(),
    mintedTimestamp: t.integer(),
    streak: t.integer(),
  }),
  (table) => ({
    strengthIndex: index().on(table.strength),
  })
);

export const Contribution = onchainTable("contribution", (t) => ({
  id: t.text().primaryKey(),
  accountId: t.text().notNull(),
  canvasId: t.integer().notNull(),
  pixelsCount: t.integer(),
}));

export const Usage = onchainTable("usage", (t) => ({
  id: t.text().primaryKey(),
  brushId: t.integer().notNull(),
  canvasId: t.integer().notNull(),
  pixelsCount: t.integer(),
}));

export const Stroke = onchainTable(
  "stroke",
  (t) => ({
    id: t.bigint().primaryKey(),
    canvasId: t.integer().notNull(),
    accountId: t.text().notNull(),
    brushId: t.integer().notNull(),
    data: t.text(),
    pixels: t.integer(),
    tx: t.text(),
    timestamp: t.integer(),
  }),
  (table) => ({
    txIndex: index().on(table.tx),
  })
);

export const Withdrawal = onchainTable("withdrawal", (t) => ({
  id: t.text().primaryKey(),
  accountId: t.text().notNull(),
  canvasId: t.integer().notNull(),
  amount: t.bigint(),
  timestamp: t.integer(),
}));

export const Canvas = onchainTable("canvas", (t) => ({
  id: t.integer().primaryKey(),
  totalMints: t.integer(),
  totalBurns: t.integer(),
  totalEarned: t.bigint(),
  totalArtists: t.integer(),
  pixelsCount: t.integer(),
  name: t.text(),
  palette: t.text(),
  size: t.integer(),
  proposer: t.text(),
}));

export const Animation = onchainTable("animation", (t) => ({
  id: t.integer().primaryKey(),
  totalMints: t.integer(),
}));

export const Balance = onchainTable(
  "balance",
  (t) => ({
    id: t.text().primaryKey(),
    ownerId: t.text().notNull(),
    contract: t.text(),
    tokenId: t.bigint(),
    value: t.integer(),
  }),
  (table) => ({
    ownerIndex: index().on(table.ownerId),
    contractIndex: index().on(table.contract),
    tokenIdIndex: index().on(table.tokenId),
  })
);

export const TotalBalance = onchainTable(
  "total_balance",
  (t) => ({
    id: t.text().primaryKey(),
    ownerId: t.text().notNull(),
    contract: t.text(),
    value: t.integer(),
  }),
  (table) => ({
    ownerIndex: index().on(table.ownerId),
    contractIndex: index().on(table.contract),
  })
);
