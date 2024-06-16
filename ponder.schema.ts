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

    brushes: p.many("Brush.ownerId"),
    contributions: p.many("Contribution.accountId"),
    withdrawals: p.many("Withdrawal.accountId"),
    strokes: p.many("Stroke.accountId"),
  }),

  Brush: p.createTable({
    id: p.int(),
    ownerId: p.string().references("Account.id"),
    strength: p.int(),
    strengthRemaining: p.int(),
    lastUsedTimestamp: p.int().optional(),
    lastUsedDay: p.int().optional(),
    streak: p.int(),

    owner: p.one("ownerId"),
    usages: p.many("Usage.brushId"),
    strokes: p.many("Stroke.brushId"),
  }),

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
      tx: p.string(),
      timestamp: p.int(),
      minted: p.boolean(),

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
    totalEarned: p.bigint(),
    totalArtists: p.int(),
    pixelsCount: p.int(),

    contributions: p.many("Contribution.canvasId"),
    withdrawals: p.many("Withdrawal.canvasId"),
    strokes: p.many("Stroke.canvasId"),
  }),
}));
