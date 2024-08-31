import { ponder } from "@/generated";

ponder.on("BasePaintAnimation:TransferSingle", async ({ event, context }) => {
  const { Animation, Canvas } = context.db;

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
  }
});
