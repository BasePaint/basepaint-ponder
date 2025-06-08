import { ponder } from "@/generated";
import { trackBalance } from "./utils";

ponder.on("BasePaintAnimation:TransferSingle", async ({ event, context }) => {
  await trackBalance(context.contracts.BasePaintAnimation.address, event, context);

  const { Animation, Canvas, Global } = context.db;

  if (BigInt(event.args.from) === 0n) {
    const count = Number(event.args.value);

    await Animation.upsert({
      id: Number(event.args.id),
      create: {
        totalMints: count,
      },
      update: ({ current }) => ({
        totalMints: current.totalMints + count,
      }),
    });

    await Canvas.update({
      id: Number(event.args.id),
      data: ({ current }) => ({
        totalBurns: current.totalBurns + 2 * count,
      }),
    });

    await Global.update({
      id: 1,
      data: ({ current }) => ({
        totalBurns: current.totalBurns + 2 * count,
      }),
    });
  }
});
