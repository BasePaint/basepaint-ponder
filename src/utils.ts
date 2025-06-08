import { Context, ponder } from "@/generated";

interface TransferSingleEvent {
  name: "TransferSingle";
  args: {
    operator: `0x${string}`;
    from: `0x${string}`;
    to: `0x${string}`;
    id: bigint;
    value: bigint;
  };
}

interface TransferBatchEvent {
  name: "TransferBatch";
  args: {
    operator: `0x${string}`;
    from: `0x${string}`;
    to: `0x${string}`;
    ids: readonly bigint[];
    values: readonly bigint[];
  };
}

interface TransferEvent {
  name: "Transfer";
  args: {
    from: `0x${string}`;
    to: `0x${string}`;
    tokenId: bigint;
  };
}

export async function trackBalance(
  contract: `0x${string}`,
  event: TransferSingleEvent | TransferBatchEvent | TransferEvent,
  context: Context
) {
  const { Balance } = context.db;

  // prettier-ignore
  const ids = event.name === "TransferSingle" ? [event.args.id] : event.name === "Transfer" ? [event.args.tokenId] : event.args.ids;
  // prettier-ignore
  const values = event.name === "TransferSingle" ? [Number(event.args.value)] : event.name === "Transfer" ? [1] : event.args.values;

  for (let i = 0; i < ids.length; i++) {
    const id = ids[i]!;
    const value = Number(values[i]!);

    const fromId = `${contract}_${event.args.from}_${id}`;
    const toId = `${contract}_${event.args.to}_${id}`;

    if (event.args.from !== "0x0000000000000000000000000000000000000000") {
      await Balance.update({
        id: fromId,
        data: ({ current }) => ({
          value: current.value - value,
        }),
      });
    }

    if (event.args.to !== "0x0000000000000000000000000000000000000000") {
      await Balance.upsert({
        id: toId,
        create: {
          ownerId: event.args.to,
          contract,
          tokenId: id,
          value,
        },
        update: ({ current }) => ({
          value: current.value + value,
        }),
      });
    }
  }
}
