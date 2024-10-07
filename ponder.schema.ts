import { createSchema } from "@ponder/core";

export default createSchema((p) => ({
  Global: p.createTable({
    id: p.int(),
    startedAt: p.int(),
    epochDuration: p.int(),
  }),

  Account: p.createTable({
    id: p.string(),
    totalPixels: p.int(),
    totalWithdrawn: p.bigint(),
    totalEarned: p.bigint(),
    streak: p.int(),
    longestStreak: p.int(),
    lastPaintedDay: p.int().optional(),

    brushes: p.many("Brush.ownerId"),
    contributions: p.many("Contribution.accountId"),
    withdrawals: p.many("Withdrawal.accountId"),
    strokes: p.many("Stroke.accountId"),
  }),

  Brush: p.createTable(
    {
      id: p.int(),
      ownerId: p.string().references("Account.id"),
      strength: p.int(),
      lastUsedTimestamp: p.int().optional(),
      lastUsedDay: p.int().optional(),
      mintedTimestamp: p.int(),
      streak: p.int(),

      owner: p.one("ownerId"),
      usages: p.many("Usage.brushId"),
      strokes: p.many("Stroke.brushId"),
    },
    {
      strengthIndex: p.index("strength"),
    }
  ),

  Contribution: p.createTable({
    id: p.string(),
    accountId: p.string().references("Account.id"),
    canvasId: p.int().references("Canvas.id"),
    pixelsCount: p.int(),

    // TODO: add strokes?
    account: p.one("accountId"),
    canvas: p.one("canvasId"),
  }),

  Usage: p.createTable({
    id: p.string(),
    brushId: p.int().references("Brush.id"),
    canvasId: p.int().references("Canvas.id"),
    pixelsCount: p.int(),

    brush: p.one("brushId"),
    canvas: p.one("canvasId"),
  }),

  Stroke: p.createTable(
    {
      id: p.bigint(),
      canvasId: p.int().references("Canvas.id"),
      accountId: p.string().references("Account.id"),
      brushId: p.int().references("Brush.id"),
      data: p.string(),
      pixels: p.int(),
      tx: p.string(),
      timestamp: p.int(),

      canvas: p.one("canvasId"),
      account: p.one("accountId"),
      brush: p.one("brushId"),
    },
    {
      txIndex: p.index("tx"),
    }
  ),

  Withdrawal: p.createTable({
    id: p.string(),
    accountId: p.string().references("Account.id"),
    canvasId: p.int().references("Canvas.id"),
    amount: p.bigint(),
    timestamp: p.int(),

    canvas: p.one("canvasId"),
    account: p.one("accountId"),
  }),

  Canvas: p.createTable({
    id: p.int(),
    totalMints: p.int(),
    totalBurns: p.int(),
    totalEarned: p.bigint(),
    totalArtists: p.int(),
    pixelsCount: p.int(),
    name: p.string().optional(),
    palette: p.string().optional(),
    size: p.int().optional(),
    proposer: p.string().optional(),

    contributions: p.many("Contribution.canvasId"),
    withdrawals: p.many("Withdrawal.canvasId"),
    strokes: p.many("Stroke.canvasId"),
  }),

  Animation: p.createTable({
    id: p.int(),
    totalMints: p.int(),
  }),
}));
