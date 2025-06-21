import { onchainTable, index, relations } from "ponder";

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

export const AccountRelations = relations(Account, ({ many }) => ({
  brushes: many(Brush),
  contributions: many(Contribution),
  strokes: many(Stroke),
  withdrawals: many(Withdrawal),
  balances: many(Balance),
  totalBalances: many(TotalBalance),
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

export const BrushRelations = relations(Brush, ({ one, many }) => ({
  owner: one(Account, { fields: [Brush.ownerId], references: [Account.id] }),
  usages: many(Usage),
  strokes: many(Stroke),
}));

export const Contribution = onchainTable("contribution", (t) => ({
  id: t.text().primaryKey(),
  accountId: t.text().notNull(),
  canvasId: t.integer().notNull(),
  pixelsCount: t.integer(),
}));

export const ContributionRelations = relations(Contribution, ({ one }) => ({
  account: one(Account, { fields: [Contribution.accountId], references: [Account.id] }),
  canvas: one(Canvas, { fields: [Contribution.canvasId], references: [Canvas.id] }),
}));

export const Usage = onchainTable("usage", (t) => ({
  id: t.text().primaryKey(),
  brushId: t.integer().notNull(),
  canvasId: t.integer().notNull(),
  pixelsCount: t.integer(),
}));

export const UsageRelations = relations(Usage, ({ one }) => ({
  brush: one(Brush, { fields: [Usage.brushId], references: [Brush.id] }),
  canvas: one(Canvas, { fields: [Usage.canvasId], references: [Canvas.id] }),
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

export const StrokeRelations = relations(Stroke, ({ one }) => ({
  canvas: one(Canvas, { fields: [Stroke.canvasId], references: [Canvas.id] }),
  account: one(Account, { fields: [Stroke.accountId], references: [Account.id] }),
  brush: one(Brush, { fields: [Stroke.brushId], references: [Brush.id] }),
}));

export const Withdrawal = onchainTable("withdrawal", (t) => ({
  id: t.text().primaryKey(),
  accountId: t.text().notNull(),
  canvasId: t.integer().notNull(),
  amount: t.bigint(),
  timestamp: t.integer(),
}));

export const WithdrawalRelations = relations(Withdrawal, ({ one }) => ({
  account: one(Account, { fields: [Withdrawal.accountId], references: [Account.id] }),
  canvas: one(Canvas, { fields: [Withdrawal.canvasId], references: [Canvas.id] }),
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

export const CanvasRelations = relations(Canvas, ({ many }) => ({
  contributions: many(Contribution),
  usages: many(Usage),
  strokes: many(Stroke),
  withdrawals: many(Withdrawal),
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

export const BalanceRelations = relations(Balance, ({ one }) => ({
  owner: one(Account, { fields: [Balance.ownerId], references: [Account.id] }),
}));

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

export const TotalBalanceRelations = relations(TotalBalance, ({ one }) => ({
  owner: one(Account, { fields: [TotalBalance.ownerId], references: [Account.id] }),
}));
