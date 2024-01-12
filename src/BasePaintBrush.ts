import { ponder } from "@/generated";

ponder.on("BasePaintBrush:Transfer", async ({ event, context }) => {
  const { Brush, Account } = context.db;
  const { BasePaintBrush } = context.contracts;

  const strength = Number(
    await context.client.readContract({
      abi: BasePaintBrush.abi,
      address: BasePaintBrush.address,
      functionName: "strengths",
      args: [event.args.tokenId],
    })
  );

  await Brush.upsert({
    id: Number(event.args.tokenId),
    create: {
      ownerId: event.args.to,
      strength,
      streak: 0,
      strengthRemaining: strength,
    },
    update: {
      ownerId: event.args.to,
      strength,
    },
  });

  const account = await Account.findUnique({ id: event.args.to });
  if (!account) {
    await Account.create({
      id: event.args.to,
      data: {
        totalPixels: 0,
      },
    });
  }
});
