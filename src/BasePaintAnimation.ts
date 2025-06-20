import { ponder } from "ponder:registry";
import { Animation, Canvas, Global } from "ponder:schema";

ponder.on("BasePaintAnimation:TransferSingle", async ({ event, context }) => {
  if (BigInt(event.args.from) === 0n) {
    const count = Number(event.args.value);

    await context.db
      .insert(Animation)
      .values({
        id: Number(event.args.id),
        totalMints: count,
      })
      .onConflictDoUpdate((row) => ({
        totalMints: (row.totalMints ?? 0) + count,
      }));

    const canvas = await context.db.find(Canvas, { id: Number(event.args.id) });
    if (canvas) {
      await context.db.update(Canvas, { id: Number(event.args.id) }).set({
        totalBurns: (canvas.totalBurns ?? 0) + 2 * count,
      });
    }

    const global = await context.db.find(Global, { id: 1 });
    if (global) {
      await context.db.update(Global, { id: 1 }).set({
        totalBurns: (global.totalBurns ?? 0) + 2 * count,
      });
    }
  }
});
