import { ponder } from "ponder:registry";
import { Brush, Account } from "ponder:schema";

ponder.on("BasePaintBrush:Transfer", async ({ event, context }) => {
  await context.db
    .insert(Brush)
    .values({
      id: Number(event.args.tokenId),
      ownerId: event.args.to,
      strength: 0,
      streak: 0,
      mintedTimestamp: Number(event.block.timestamp),
    })
    .onConflictDoUpdate((row) => ({
      ownerId: event.args.to,
    }));

  await context.db
    .insert(Account)
    .values({
      id: event.args.to,
      totalPixels: 0,
      totalWithdrawn: 0n,
      totalEarned: 0n,
      streak: 0,
      longestStreak: 0,
    })
    .onConflictDoUpdate((row) => ({}));
});

ponder.on("BasePaintBrushEvents:Deployed", async ({ event, context }) => {
  const { BasePaintBrush } = context.contracts;

  // Reindex all brush strengths to get accurate data for brushes
  // before the BasePaintBrushEvents contract.
  const brushes = await context.db.sql.select().from(Brush);

  const strengths = await Promise.all(
    brushes.map((brush) =>
      context.client.readContract({
        abi: BasePaintBrush.abi,
        address: BasePaintBrush.address,
        functionName: "strengths",
        args: [BigInt(brush.id)],
      })
    )
  );

  for (const [index, brush] of brushes.entries()) {
    await context.db.update(Brush, { id: brush.id }).set({
      strength: Number(strengths[index]),
    });
  }

  // From now on, we will listen to StrengthChanged events
});

ponder.on("BasePaintBrushEvents:StrengthChanged", async ({ event, context }) => {
  await context.db.update(Brush, { id: Number(event.args.tokenId) }).set({
    strength: Number(event.args.strength),
  });
});
