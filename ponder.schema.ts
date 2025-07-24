import { onchainTable, index, relations } from "ponder";

export const Global = onchainTable("global", (t) => ({
  id: t.integer().primaryKey(),
  startedAt: t.integer().notNull(),
  epochDuration: t.integer().notNull(),
  totalArtists: t.integer().notNull(),
  totalPixels: t.integer().notNull(),
  totalEarnings: t.bigint().notNull(),
  totalWithdrawals: t.bigint().notNull(),
  totalMints: t.integer().notNull(),
  totalBurns: t.integer().notNull(),
  totalSubscriptions: t.integer().notNull(),
}));

export const Account = onchainTable("account", (t) => ({
  id: t.text().primaryKey(),
  totalPixels: t.integer().notNull(),
  totalWithdrawn: t.bigint().notNull(),
  totalEarned: t.bigint().notNull(),
  streak: t.integer().notNull(),
  longestStreak: t.integer().notNull(),
  lastPaintedDay: t.integer(),
  totalDaysPainted: t.integer().notNull().default(0),
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
    strength: t.integer().notNull(),
    lastUsedTimestamp: t.integer(),
    lastUsedDay: t.integer(),
    mintedTimestamp: t.integer().notNull(),
    streak: t.integer().notNull(),
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
  pixelsCount: t.integer().notNull(),
}));

export const ContributionRelations = relations(Contribution, ({ one }) => ({
  account: one(Account, { fields: [Contribution.accountId], references: [Account.id] }),
  canvas: one(Canvas, { fields: [Contribution.canvasId], references: [Canvas.id] }),
}));

export const Usage = onchainTable("usage", (t) => ({
  id: t.text().primaryKey(),
  brushId: t.integer().notNull(),
  canvasId: t.integer().notNull(),
  pixelsCount: t.integer().notNull(),
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
    data: t.text().notNull(),
    pixels: t.integer().notNull(),
    tx: t.text().notNull(),
    timestamp: t.integer().notNull(),
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
  amount: t.bigint().notNull(),
  timestamp: t.integer().notNull(),
}));

export const WithdrawalRelations = relations(Withdrawal, ({ one }) => ({
  account: one(Account, { fields: [Withdrawal.accountId], references: [Account.id] }),
  canvas: one(Canvas, { fields: [Withdrawal.canvasId], references: [Canvas.id] }),
}));

export const Canvas = onchainTable("canvas", (t) => ({
  id: t.integer().primaryKey(),
  totalMints: t.integer().notNull(),
  totalBurns: t.integer().notNull(),
  totalEarned: t.bigint().notNull(),
  totalArtists: t.integer().notNull(),
  pixelsCount: t.integer().notNull(),
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
  totalMints: t.integer().notNull(),
}));

export const Balance = onchainTable(
  "balance",
  (t) => ({
    id: t.text().primaryKey(),
    ownerId: t.text().notNull(),
    contract: t.text().notNull(),
    tokenId: t.bigint().notNull(),
    value: t.integer().notNull(),
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
    contract: t.text().notNull(),
    value: t.integer().notNull(),
  }),
  (table) => ({
    ownerIndex: index().on(table.ownerId),
    contractIndex: index().on(table.contract),
  })
);

export const TotalBalanceRelations = relations(TotalBalance, ({ one }) => ({
  owner: one(Account, { fields: [TotalBalance.ownerId], references: [Account.id] }),
}));
