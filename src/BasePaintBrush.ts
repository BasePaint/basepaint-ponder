import { ponder } from "ponder:registry";
import { Brush, Account } from "ponder:schema";
import { trackBalance } from "./utils";
import { checksumAddress, isZeroAddress } from "./address";
import { BASE_PAINT_BRUSH_EVENTS_DEPLOYED_BLOCK } from "../constants";

ponder.on("BasePaintBrush:Transfer", async ({ event, context }) => {
  // Track balance changes
  await trackBalance("0xD68fe5b53e7E1AbeB5A4d0A6660667791f39263a", event, context);

  const tokenId = Number(event.args.tokenId);
  const owner = checksumAddress(event.args.to);
  const shouldIndexMintStrength =
    isZeroAddress(event.args.from) &&
    event.block.number >= BigInt(BASE_PAINT_BRUSH_EVENTS_DEPLOYED_BLOCK);
  const strength = shouldIndexMintStrength
    ? Number(
        await context.client.readContract({
          abi: context.contracts.BasePaintBrush.abi,
          address: context.contracts.BasePaintBrush.address,
          functionName: "strengths",
          args: [event.args.tokenId],
          blockNumber: event.block.number,
        })
      )
    : 0;

  await context.db
    .insert(Brush)
    .values({
      id: tokenId,
      ownerId: owner,
      strength,
      streak: 0,
      mintedTimestamp: Number(event.block.timestamp),
    })
    .onConflictDoUpdate((row) => ({
      ownerId: owner,
    }));

  await context.db
    .insert(Account)
    .values({
      id: owner,
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
